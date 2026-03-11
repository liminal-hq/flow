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
use crate::state::{Mode, TuiState};
use crate::ui::{help, input_pane, layout, reply_pane, thread_list};

const TICK_RATE: Duration = Duration::from_millis(250);

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
            input_pane::render(frame, app_layout.input_pane, &textarea, state.mode);

            if state.mode == Mode::Help {
                help::render(frame, frame.area());
            }
        })?;

        if state.should_quit {
            return Ok(());
        }

        // Poll for crossterm events with tick timeout
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }

                match state.mode {
                    Mode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
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
                            state.mode = Mode::Help;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            state.select_next();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            state.select_prev();
                        }
                        _ => {}
                    },

                    Mode::Insert => match key.code {
                        KeyCode::Esc => {
                            state.mode = Mode::Normal;
                        }
                        KeyCode::Enter => {
                            // Submit the input
                            let lines: Vec<String> = textarea.lines().to_vec();
                            let text = lines.join("\n");

                            // Clear the textarea
                            textarea = TextArea::default();
                            textarea
                                .set_cursor_line_style(ratatui::style::Style::default());

                            // Process the input
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
                            state.poll_watermark = poll::current_watermark(conn);
                        }
                        _ => {
                            // Forward to textarea
                            textarea.input(Event::Key(key));
                        }
                    },
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
