// Main TUI event loop
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
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
use crate::state::filtered_slash_commands;
use crate::state::{Mode, TuiState};
use crate::ui::{
    about, command_palette, help, hints_bar, input_pane, layout, reply_pane, thread_list,
};

const TICK_RATE: Duration = Duration::from_millis(250);

fn should_follow_active_after_submit(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed == "/resume"
        || trimmed == "/park"
        || matches!(
            input::parsed_intent(input),
            Some(
                liminal_flow_core::model::Intent::SetCurrentThread
                    | liminal_flow_core::model::Intent::StartBranch
                    | liminal_flow_core::model::Intent::ReturnToParent
                    | liminal_flow_core::model::Intent::AddNote
            )
        )
        || (!trimmed.is_empty() && !trimmed.starts_with('/'))
}

/// Run the TUI application. Takes ownership of the database connection.
pub fn run(conn: Connection) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Install panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    let result = run_loop(&mut terminal, &conn);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &Connection,
) -> Result<()> {
    fn terminal_area(
        terminal: &Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<ratatui::layout::Rect> {
        let size = terminal.size()?;
        Ok(ratatui::layout::Rect::new(0, 0, size.width, size.height))
    }

    fn thread_viewport_height(terminal: &Terminal<CrosstermBackend<io::Stdout>>) -> Result<usize> {
        let app_layout = layout::compute(terminal_area(terminal)?);
        Ok(app_layout.thread_list.height.saturating_sub(2) as usize)
    }

    fn sync_thread_viewport(
        terminal: &Terminal<CrosstermBackend<io::Stdout>>,
        state: &mut TuiState,
    ) -> Result<()> {
        let viewport_height = thread_viewport_height(terminal)?;
        state.ensure_thread_selection_visible(viewport_height);
        state.clamp_thread_list_scroll(viewport_height);
        Ok(())
    }

    let mut state = TuiState::new();
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(ratatui::style::Style::default());

    // Initial load
    state.refresh_from_db(conn);
    sync_thread_viewport(terminal, &mut state)?;
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
                let query = textarea.lines().join("\n");
                command_palette::render(frame, app_layout.input_pane, &state, &query);
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
            match event::read()? {
                Event::Mouse(mouse) => {
                    let terminal_area = terminal_area(terminal)?;
                    let app_layout = layout::compute(terminal_area);

                    if state.mode == Mode::Help {
                        let popup_area = help::popup_area(terminal_area);
                        if layout::contains_point(popup_area, mouse.column, mouse.row) {
                            match mouse.kind {
                                MouseEventKind::ScrollUp => {
                                    state.help_scroll = state.help_scroll.saturating_sub(3);
                                }
                                MouseEventKind::ScrollDown => {
                                    state.help_scroll = state.help_scroll.saturating_add(3);
                                }
                                _ => {}
                            }
                        }
                        continue;
                    }

                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if layout::contains_point(
                                app_layout.thread_list,
                                mouse.column,
                                mouse.row,
                            ) {
                                state.scroll_thread_list(
                                    -3,
                                    app_layout.thread_list.height.saturating_sub(2) as usize,
                                );
                            } else if layout::contains_point(
                                app_layout.reply_pane,
                                mouse.column,
                                mouse.row,
                            ) {
                                state.status_scroll = state.status_scroll.saturating_sub(3);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if layout::contains_point(
                                app_layout.thread_list,
                                mouse.column,
                                mouse.row,
                            ) {
                                state.scroll_thread_list(
                                    3,
                                    app_layout.thread_list.height.saturating_sub(2) as usize,
                                );
                            } else if layout::contains_point(
                                app_layout.reply_pane,
                                mouse.column,
                                mouse.row,
                            ) {
                                state.status_scroll = state.status_scroll.saturating_add(3);
                            }
                        }
                        MouseEventKind::Down(MouseButton::Left) => {
                            if layout::contains_point(
                                app_layout.thread_list,
                                mouse.column,
                                mouse.row,
                            ) && mouse.row > app_layout.thread_list.y
                                && mouse.row
                                    < app_layout
                                        .thread_list
                                        .y
                                        .saturating_add(app_layout.thread_list.height)
                                        .saturating_sub(1)
                            {
                                let row_offset =
                                    mouse.row.saturating_sub(app_layout.thread_list.y + 1) as usize;
                                let visible_rows = state.visible_rows();
                                let row_index = usize::from(state.thread_list_scroll)
                                    .saturating_add(row_offset);
                                if let Some(selected) = visible_rows.get(row_index) {
                                    state.selected = selected.clone();
                                    state.refresh_selected_details(conn);
                                    sync_thread_viewport(terminal, &mut state)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Key(key) => {
                    // Ctrl+C always quits
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
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
                                state.refresh_selected_details(conn);
                                sync_thread_viewport(terminal, &mut state)?;
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
                                                input::resume_branch(
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
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('p') => {
                                let result = match &state.selected {
                                    crate::state::SelectedItem::Thread(_) => None,
                                    crate::state::SelectedItem::Branch(i, j) => {
                                        state.threads.get(*i).and_then(|entry| {
                                            entry.branches.get(*j).map(|branch| {
                                                input::park_branch(
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
                                        InputResult::Reply(msg) => {
                                            state.last_reply = Some(msg);
                                            state.error_message = None;
                                            if let crate::state::SelectedItem::Branch(i, _) =
                                                state.selected
                                            {
                                                state.selected =
                                                    crate::state::SelectedItem::Thread(i);
                                            }
                                        }
                                        InputResult::Error(msg) => {
                                            state.error_message = Some(msg);
                                        }
                                        InputResult::None => {}
                                    }
                                    state.refresh_from_db(conn);
                                    state.select_active_item();
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('d') => {
                                let result = match &state.selected {
                                    crate::state::SelectedItem::Thread(i) => {
                                        state.threads.get(*i).map(|entry| {
                                            input::mark_thread_done(conn, &entry.thread.id)
                                        })
                                    }
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
                                    sync_thread_viewport(terminal, &mut state)?;
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
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                state.select_next();
                                state.refresh_selected_details(conn);
                                sync_thread_viewport(terminal, &mut state)?;
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                state.select_prev();
                                state.refresh_selected_details(conn);
                                sync_thread_viewport(terminal, &mut state)?;
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
                                let clamp_palette_selection =
                                    |state: &mut TuiState, query: &str| {
                                        let count = filtered_slash_commands(query).len();
                                        if count == 0 {
                                            state.command_palette_index = 0;
                                        } else if state.command_palette_index >= count {
                                            state.command_palette_index = count - 1;
                                        }
                                    };

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
                                        let query = textarea.lines().join("\n");
                                        let count = filtered_slash_commands(&query).len();
                                        if count == 0 {
                                            state.command_palette_index = 0;
                                        } else if state.command_palette_index > 0 {
                                            state.command_palette_index -= 1;
                                        } else {
                                            state.command_palette_index = count - 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        let query = textarea.lines().join("\n");
                                        let count = filtered_slash_commands(&query).len();
                                        if count > 0 {
                                            state.command_palette_index =
                                                (state.command_palette_index + 1) % count;
                                        }
                                    }
                                    KeyCode::Enter | KeyCode::Tab => {
                                        let query = textarea.lines().join("\n");
                                        let filtered = filtered_slash_commands(&query);
                                        if let Some((_, cmd, _)) =
                                            filtered.get(state.command_palette_index)
                                        {
                                            let cmd_name =
                                                cmd.split_whitespace().next().unwrap_or(cmd);
                                            textarea = TextArea::default();
                                            textarea.set_cursor_line_style(
                                                ratatui::style::Style::default(),
                                            );
                                            textarea.insert_str(format!("{cmd_name} "));
                                            state.show_command_palette = false;
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        textarea.input(Event::Key(key));
                                        let query = textarea.lines().join("\n");
                                        if query.trim().is_empty()
                                            || !query.trim_start().starts_with('/')
                                        {
                                            state.show_command_palette = false;
                                        } else {
                                            clamp_palette_selection(&mut state, &query);
                                        }
                                    }
                                    KeyCode::Char(_) => {
                                        textarea.input(Event::Key(key));
                                        let query = textarea.lines().join("\n");
                                        if !query.trim_start().starts_with('/') {
                                            state.show_command_palette = false;
                                        } else {
                                            clamp_palette_selection(&mut state, &query);
                                        }
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
                                        state.refresh_selected_details(conn);
                                        sync_thread_viewport(terminal, &mut state)?;
                                    }
                                    KeyCode::Down => {
                                        // Arrow keys navigate the thread list
                                        state.select_next();
                                        state.refresh_selected_details(conn);
                                        sync_thread_viewport(terminal, &mut state)?;
                                    }
                                    KeyCode::Enter => {
                                        // If input is empty, toggle thread expansion
                                        let is_empty =
                                            textarea.lines().iter().all(|l| l.is_empty());
                                        if is_empty {
                                            state.toggle_expanded();
                                            state.refresh_selected_details(conn);
                                            sync_thread_viewport(terminal, &mut state)?;
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
                                        let follow_active =
                                            should_follow_active_after_submit(&text);
                                        if text.trim() == "/resume" {
                                            let result = match &state.selected {
                                                crate::state::SelectedItem::Thread(i) => {
                                                    state.threads.get(*i).map(|entry| {
                                                        input::resume_thread(conn, &entry.thread.id)
                                                    })
                                                }
                                                crate::state::SelectedItem::Branch(i, j) => {
                                                    state.threads.get(*i).and_then(|entry| {
                                                        entry.branches.get(*j).map(|branch| {
                                                            input::resume_branch(
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
                                                    InputResult::Reply(msg) => {
                                                        state.last_reply = Some(msg);
                                                        state.error_message = None;
                                                    }
                                                    InputResult::Error(msg) => {
                                                        state.error_message = Some(msg);
                                                    }
                                                    InputResult::None => {}
                                                }
                                            }
                                        } else if text.trim() == "/park" {
                                            let result = state.active_thread().and_then(|entry| {
                                                state.active_branch().map(|branch| {
                                                    input::park_branch(
                                                        conn,
                                                        &entry.thread.id,
                                                        &branch.id,
                                                    )
                                                })
                                            });
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
                                            } else {
                                                state.error_message =
                                                    Some("No active branch to park.".into());
                                            }
                                        } else if text.trim() == "/archive" {
                                            let result = if let Some(active_branch) =
                                                state.active_branch()
                                            {
                                                state.active_thread().map(|entry| {
                                                    input::archive_branch(
                                                        conn,
                                                        &entry.thread.id,
                                                        &active_branch.id,
                                                    )
                                                })
                                            } else {
                                                state.active_thread().map(|entry| {
                                                    input::archive_thread(conn, &entry.thread.id)
                                                })
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
                                            } else {
                                                state.error_message = Some(
                                                    "No active thread or branch to archive.".into(),
                                                );
                                            }
                                        } else {
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
                                        }

                                        // Refresh state from DB after mutation
                                        state.refresh_from_db(conn);
                                        if follow_active {
                                            state.select_active_item();
                                        }
                                        sync_thread_viewport(terminal, &mut state)?;
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
                _ => {}
            }
        }

        // Check for external DB changes (from CLI in another terminal)
        if poll::has_changes(conn, &state.poll_watermark) {
            state.refresh_from_db(conn);
            sync_thread_viewport(terminal, &mut state)?;
            state.poll_watermark = poll::current_watermark(conn);
        }
    }
}
