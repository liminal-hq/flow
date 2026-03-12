// Main TUI event loop
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rusqlite::Connection;
use tui_textarea::TextArea;

use crate::input::{self, InputResult};
use crate::poll;
use crate::state::SLASH_COMMANDS;
use crate::state::{Mode, TuiState};
use crate::ui::{
    about, command_palette, help, hints_bar, input_pane, layout, reply_pane, thread_list,
};

const TICK_RATE: Duration = Duration::from_millis(250);

fn should_follow_active_after_submit(input: &str) -> bool {
    matches!(
        input::parsed_intent(input),
        Some(
            liminal_flow_core::model::Intent::SetCurrentThread
                | liminal_flow_core::model::Intent::StartBranch
                | liminal_flow_core::model::Intent::ReturnToParent
        )
    )
}

/// Run the TUI application. Takes ownership of the database connection.
pub fn run(conn: Connection) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Install panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let result = run_loop(&mut terminal, &conn);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &Connection,
) -> Result<()> {
    let mut state = TuiState::new();
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(ratatui::style::Style::default());

    // Initial load
    state.refresh_from_db(conn);
    state.poll_watermark = poll::current_watermark(conn);

    loop {
        // Draw
        terminal.draw(|frame| {
            let app_layout = layout::compute(frame.area());

            layout::render_header(frame, app_layout.header);
            thread_list::render(frame, app_layout.thread_list, &state);
            reply_pane::render(frame, app_layout.reply_pane, &state);
            let active_target = state.active_capture_target_label();
            input_pane::render(
                frame,
                app_layout.input_pane,
                &textarea,
                state.mode,
                active_target.as_deref(),
            );

            // Floating overlays above the input pane
            if state.show_command_palette {
                command_palette::render(frame, app_layout.input_pane, &state);
            } else if state.show_hints {
                hints_bar::render(frame, app_layout.input_pane);
            }

            if state.mode == Mode::Help {
                help::render(frame, frame.area(), state.help_scroll);
            } else if state.mode == Mode::About {
                about::render(frame, frame.area());
            }
        })?;

        if state.should_quit {
            return Ok(());
        }

        // Poll for crossterm events with tick timeout
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                match state.mode {
                    Mode::Help => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.help_scroll = state.help_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.help_scroll = state.help_scroll.saturating_add(1);
                        }
                        KeyCode::PageUp => {
                            state.help_scroll = state.help_scroll.saturating_sub(8);
                        }
                        KeyCode::PageDown => {
                            state.help_scroll = state.help_scroll.saturating_add(8);
                        }
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                            state.mode = Mode::Normal;
                        }
                        _ => {}
                    },

                    Mode::About => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                            state.mode = Mode::Normal;
                        }
                        _ => {}
                    },

                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            state.should_quit = true;
                        }
                        KeyCode::Char('i') => {
                            state.mode = Mode::Insert;
                        }
                        KeyCode::Char('?') => {
                            state.help_scroll = 0;
                            state.mode = Mode::Help;
                        }
                        KeyCode::Char('a') => {
                            state.mode = Mode::About;
                        }
                        KeyCode::Enter => {
                            state.toggle_expanded();
                        }
                        KeyCode::Char('r') => {
                            // Resume/activate the selected thread or branch
                            let result = match &state.selected {
                                crate::state::SelectedItem::Thread(i) => state
                                    .threads
                                    .get(*i)
                                    .map(|entry| input::resume_thread(conn, &entry.thread.id)),
                                crate::state::SelectedItem::Branch(i, j) => {
                                    state.threads.get(*i).and_then(|entry| {
                                        entry.branches.get(*j).map(|branch| {
                                            input::resume_branch(conn, &entry.thread.id, &branch.id)
                                        })
                                    })
                                }
                            };
                            if let Some(result) = result {
                                match result {
                                    InputResult::Reply(msg) => {
                                        state.last_reply = Some(msg);
                                        state.error_message = None;
                                    }
                                    InputResult::Error(msg) => {
                                        state.error_message = Some(msg);
                                    }
                                    InputResult::None => {}
                                }
                                state.refresh_from_db(conn);
                                state.select_active_item();
                                state.poll_watermark = poll::current_watermark(conn);
                            }
                        }
                        KeyCode::Char('p') => {
                            let result = match &state.selected {
                                crate::state::SelectedItem::Thread(_) => None,
                                crate::state::SelectedItem::Branch(i, j) => {
                                    state.threads.get(*i).and_then(|entry| {
                                        entry.branches.get(*j).map(|branch| {
                                            input::park_branch(conn, &entry.thread.id, &branch.id)
                                        })
                                    })
                                }
                            };
                            if let Some(result) = result {
                                match result {
                                    InputResult::Reply(msg) => {
                                        state.last_reply = Some(msg);
                                        state.error_message = None;
                                        if let crate::state::SelectedItem::Branch(i, _) =
                                            state.selected
                                        {
                                            state.selected = crate::state::SelectedItem::Thread(i);
                                        }
                                    }
                                    InputResult::Error(msg) => {
                                        state.error_message = Some(msg);
                                    }
                                    InputResult::None => {}
                                }
                                state.refresh_from_db(conn);
                                state.select_active_item();
                                state.poll_watermark = poll::current_watermark(conn);
                            }
                        }
                        KeyCode::Char('d') => {
                            let result = match &state.selected {
                                crate::state::SelectedItem::Thread(i) => state
                                    .threads
                                    .get(*i)
                                    .map(|entry| input::mark_thread_done(conn, &entry.thread.id)),
                                crate::state::SelectedItem::Branch(i, j) => {
                                    state.threads.get(*i).and_then(|entry| {
                                        entry.branches.get(*j).map(|branch| {
                                            input::mark_branch_done(
                                                conn,
                                                &entry.thread.id,
                                                &branch.id,
                                            )
                                        })
                                    })
                                }
                            };
                            if let Some(result) = result {
                                match result {
                                    Ok(msg) => {
                                        state.last_reply = Some(msg);
                                        state.error_message = None;
                                    }
                                    Err(err) => {
                                        state.error_message = Some(err.to_string());
                                    }
                                }
                                state.refresh_from_db(conn);
                                state.poll_watermark = poll::current_watermark(conn);
                            }
                        }
                        KeyCode::Char('A') => {
                            let result = match &state.selected {
                                crate::state::SelectedItem::Thread(i) => state
                                    .threads
                                    .get(*i)
                                    .map(|entry| input::archive_thread(conn, &entry.thread.id)),
                                crate::state::SelectedItem::Branch(i, j) => {
                                    state.threads.get(*i).and_then(|entry| {
                                        entry.branches.get(*j).map(|branch| {
                                            input::archive_branch(
                                                conn,
                                                &entry.thread.id,
                                                &branch.id,
                                            )
                                        })
                                    })
                                }
                            };
                            if let Some(result) = result {
                                match result {
                                    Ok(msg) => {
                                        state.last_reply = Some(msg);
                                        state.error_message = None;
                                    }
                                    Err(err) => {
                                        state.error_message = Some(err.to_string());
                                    }
                                }
                                state.refresh_from_db(conn);
                                state.poll_watermark = poll::current_watermark(conn);
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            state.select_next();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            state.select_prev();
                        }
                        KeyCode::PageUp => {
                            state.status_scroll = state.status_scroll.saturating_sub(5);
                        }
                        KeyCode::PageDown => {
                            state.status_scroll = state.status_scroll.saturating_add(5);
                        }
                        _ => {}
                    },

                    Mode::Insert => {
                        // Helper: check if textarea is empty (single empty line)
                        let is_empty = textarea.lines().iter().all(|l| l.is_empty());

                        if state.show_command_palette {
                            // Command palette is open — handle navigation
                            match key.code {
                                KeyCode::Esc => {
                                    state.show_command_palette = false;
                                    // Clear the `/` from the textarea
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                }
                                KeyCode::Up => {
                                    if state.command_palette_index > 0 {
                                        state.command_palette_index -= 1;
                                    } else {
                                        state.command_palette_index = SLASH_COMMANDS.len() - 1;
                                    }
                                }
                                KeyCode::Down => {
                                    state.command_palette_index =
                                        (state.command_palette_index + 1) % SLASH_COMMANDS.len();
                                }
                                KeyCode::Enter | KeyCode::Tab => {
                                    // Insert the selected command into the textarea
                                    let (cmd, _) = SLASH_COMMANDS[state.command_palette_index];
                                    // Extract just the command name (e.g., "/now" from "/now <text>")
                                    let cmd_name = cmd.split_whitespace().next().unwrap_or(cmd);
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                    // Insert command text followed by a space
                                    textarea.insert_str(format!("{cmd_name} "));
                                    state.show_command_palette = false;
                                }
                                KeyCode::Backspace => {
                                    // Close palette and clear input
                                    state.show_command_palette = false;
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                }
                                KeyCode::Char(_) => {
                                    // Any other character closes the palette and types into textarea
                                    state.show_command_palette = false;
                                    textarea.input(Event::Key(key));
                                }
                                _ => {}
                            }
                        } else if state.show_hints {
                            // Hints bar is open — any key dismisses it
                            match key.code {
                                KeyCode::Esc => {
                                    state.show_hints = false;
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                }
                                KeyCode::Backspace => {
                                    state.show_hints = false;
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                }
                                _ => {
                                    state.show_hints = false;
                                    // Clear the `?` and forward the new key
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());
                                    textarea.input(Event::Key(key));
                                }
                            }
                        } else {
                            // Normal Insert mode handling
                            match key.code {
                                KeyCode::Esc => {
                                    state.mode = Mode::Normal;
                                    state.show_command_palette = false;
                                    state.show_hints = false;
                                }
                                KeyCode::Up => {
                                    // Arrow keys navigate the thread list
                                    state.select_prev();
                                }
                                KeyCode::Down => {
                                    // Arrow keys navigate the thread list
                                    state.select_next();
                                }
                                KeyCode::Enter => {
                                    // If input is empty, toggle thread expansion
                                    let is_empty = textarea.lines().iter().all(|l| l.is_empty());
                                    if is_empty {
                                        state.toggle_expanded();
                                        continue;
                                    }

                                    // Submit the input
                                    let lines: Vec<String> = textarea.lines().to_vec();
                                    let text = lines.join("\n");

                                    // Clear the textarea
                                    textarea = TextArea::default();
                                    textarea
                                        .set_cursor_line_style(ratatui::style::Style::default());

                                    // Process the input
                                    let follow_active = should_follow_active_after_submit(&text);
                                    match input::process_input(conn, &text) {
                                        InputResult::Reply(msg) => {
                                            state.last_reply = Some(msg);
                                            state.error_message = None;
                                        }
                                        InputResult::Error(msg) => {
                                            state.error_message = Some(msg);
                                        }
                                        InputResult::None => {}
                                    }

                                    // Refresh state from DB after mutation
                                    state.refresh_from_db(conn);
                                    if follow_active {
                                        state.select_active_item();
                                    }
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                                KeyCode::Char('/') if is_empty => {
                                    // Show command palette
                                    state.show_command_palette = true;
                                    state.command_palette_index = 0;
                                    textarea.input(Event::Key(key));
                                }
                                KeyCode::Char('?') if is_empty => {
                                    // Show shortcut hints
                                    state.show_hints = true;
                                    textarea.input(Event::Key(key));
                                }
                                _ => {
                                    // Forward to textarea
                                    textarea.input(Event::Key(key));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for external DB changes (from CLI in another terminal)
        if poll::has_changes(conn, &state.poll_watermark) {
            state.refresh_from_db(conn);
            state.poll_watermark = poll::current_watermark(conn);
        }
    }
}
