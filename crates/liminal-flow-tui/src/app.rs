// Main TUI event loop
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor::Show;
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
use crate::state::{filtered_slash_commands, should_keep_command_palette_open};
use crate::state::{Mode, SelectedItem, TuiState};
use crate::ui::{
    about, command_palette, help, hints_bar, input_pane, layout, reply_pane, thread_list,
};

const TICK_RATE: Duration = Duration::from_millis(250);

fn enter_tui_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn restore_tui_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    Ok(())
}

fn should_follow_active_after_submit(input: &str) -> bool {
    use liminal_flow_core::model::Intent;
    let trimmed = input.trim();
    matches!(
        input::parsed_intent(input),
        Some(
            Intent::Resume
                | Intent::Park
                | Intent::Archive
                | Intent::SetCurrentThread
                | Intent::StartBranch
                | Intent::ReturnToParent
        )
    ) || (!trimmed.is_empty() && !trimmed.starts_with('/'))
}

/// Apply an `InputResult` to TUI state (set reply or error message).
fn apply_input_result(state: &mut TuiState, result: InputResult) {
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

fn refresh_command_palette_state(state: &mut TuiState, query: &str) {
    state.show_command_palette = should_keep_command_palette_open(query);
    if state.show_command_palette {
        state.command_palette_index = 0;
    }
}

fn complete_command_palette_selection(query: &str, cmd: &str) -> String {
    let trimmed_start = query.trim_start();
    let leading_whitespace_len = query.len() - trimmed_start.len();
    let cmd_name = cmd.split_whitespace().next().unwrap_or(cmd);
    let suffix = trimmed_start
        .find(char::is_whitespace)
        .map(|index| &trimmed_start[index..])
        .unwrap_or("");

    let mut completed = String::with_capacity(
        leading_whitespace_len + cmd_name.len() + suffix.len() + usize::from(suffix.is_empty()),
    );
    completed.push_str(&query[..leading_whitespace_len]);
    completed.push_str(cmd_name);

    if suffix.is_empty() {
        completed.push(' ');
    } else {
        completed.push_str(suffix);
    }

    completed
}

fn is_suspend_key(key: crossterm::event::KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('z')
}

fn selected_command_target(state: &TuiState) -> Option<input::CommandTarget> {
    match &state.selected {
        SelectedItem::Thread(i) => state
            .threads
            .get(*i)
            .map(|entry| input::CommandTarget::Thread(entry.thread.id.clone())),
        SelectedItem::Branch(i, j) => state.threads.get(*i).and_then(|entry| {
            entry
                .branches
                .get(*j)
                .map(|branch| input::CommandTarget::Branch {
                    thread_id: entry.thread.id.clone(),
                    branch_id: branch.id.clone(),
                })
        }),
    }
}

/// Run the TUI application. Takes ownership of the database connection.
pub fn run(conn: Connection) -> Result<()> {
    // Set up terminal
    enter_tui_terminal()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Install panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_tui_terminal();
        original_hook(panic_info);
    }));

    let result = run_loop(&mut terminal, &conn);

    // Restore terminal
    restore_tui_terminal()?;
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

                    if is_suspend_key(key) {
                        restore_tui_terminal()?;
                        // SAFETY: `raise` sends SIGTSTP to the current process so the shell can
                        // suspend and later resume the TUI via `fg`.
                        unsafe {
                            libc::raise(libc::SIGTSTP);
                        }
                        enter_tui_terminal()?;
                        terminal.clear()?;
                        state.refresh_from_db(conn);
                        sync_thread_viewport(terminal, &mut state)?;
                        state.poll_watermark = poll::current_watermark(conn);
                        continue;
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
                                    SelectedItem::Thread(i) => state
                                        .threads
                                        .get(*i)
                                        .map(|entry| input::resume_thread(conn, &entry.thread.id)),
                                    SelectedItem::Branch(i, j) => {
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
                                    apply_input_result(&mut state, result);
                                    state.refresh_from_db(conn);
                                    state.select_active_item();
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('p') => {
                                // Park the selected branch
                                let result = match &state.selected {
                                    SelectedItem::Thread(_) => None,
                                    SelectedItem::Branch(i, j) => {
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
                                    // Move selection to parent thread before refresh
                                    if matches!(result, InputResult::Reply(_)) {
                                        if let SelectedItem::Branch(i, _) = state.selected {
                                            state.selected = SelectedItem::Thread(i);
                                        }
                                    }
                                    apply_input_result(&mut state, result);
                                    state.refresh_from_db(conn);
                                    state.select_active_item();
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('d') => {
                                // Mark the selected item done
                                let result: Option<InputResult> = match &state.selected {
                                    SelectedItem::Thread(i) => state.threads.get(*i).map(|entry| {
                                        input::mark_thread_done(conn, &entry.thread.id).into()
                                    }),
                                    SelectedItem::Branch(i, j) => {
                                        state.threads.get(*i).and_then(|entry| {
                                            entry.branches.get(*j).map(|branch| {
                                                input::mark_branch_done(
                                                    conn,
                                                    &entry.thread.id,
                                                    &branch.id,
                                                )
                                                .into()
                                            })
                                        })
                                    }
                                };
                                if let Some(result) = result {
                                    apply_input_result(&mut state, result);
                                    state.refresh_from_db(conn);
                                    sync_thread_viewport(terminal, &mut state)?;
                                    state.poll_watermark = poll::current_watermark(conn);
                                }
                            }
                            KeyCode::Char('A') => {
                                // Archive the selected item
                                let result: Option<InputResult> = match &state.selected {
                                    SelectedItem::Thread(i) => state.threads.get(*i).map(|entry| {
                                        input::archive_thread(conn, &entry.thread.id).into()
                                    }),
                                    SelectedItem::Branch(i, j) => {
                                        state.threads.get(*i).and_then(|entry| {
                                            entry.branches.get(*j).map(|branch| {
                                                input::archive_branch(
                                                    conn,
                                                    &entry.thread.id,
                                                    &branch.id,
                                                )
                                                .into()
                                            })
                                        })
                                    }
                                };
                                if let Some(result) = result {
                                    apply_input_result(&mut state, result);
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
                                            let completed =
                                                complete_command_palette_selection(&query, cmd);
                                            textarea = TextArea::default();
                                            textarea.set_cursor_line_style(
                                                ratatui::style::Style::default(),
                                            );
                                            textarea.insert_str(completed);
                                            state.show_command_palette = false;
                                        }
                                    }
                                    KeyCode::Backspace
                                    | KeyCode::Delete
                                    | KeyCode::Char(_) => {
                                        textarea.input(Event::Key(key));
                                        let query = textarea.lines().join("\n");
                                        if should_keep_command_palette_open(&query) {
                                            clamp_palette_selection(&mut state, &query);
                                        } else {
                                            state.show_command_palette = false;
                                        }
                                    }
                                    KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
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
                                        let command_target = selected_command_target(&state);
                                        let result = input::perform_command_on_target(
                                            conn,
                                            &text,
                                            command_target.as_ref(),
                                        );
                                        apply_input_result(&mut state, result);

                                        // Refresh state from DB after mutation
                                        state.refresh_from_db(conn);
                                        if follow_active {
                                            state.select_active_item();
                                        }
                                        sync_thread_viewport(terminal, &mut state)?;
                                        state.poll_watermark = poll::current_watermark(conn);
                                    }
                                    KeyCode::Char('?') if is_empty => {
                                        // Show shortcut hints
                                        state.show_hints = true;
                                        textarea.input(Event::Key(key));
                                    }
                                    KeyCode::Char(_) | KeyCode::Backspace => {
                                        textarea.input(Event::Key(key));
                                        let query = textarea.lines().join("\n");
                                        refresh_command_palette_state(&mut state, &query);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_open_cases_not_covered_in_state_tests() {
        // Partial unknown commands keep palette open
        assert!(should_keep_command_palette_open("/n"));
        assert!(should_keep_command_palette_open("/note-taking"));
    }

    #[test]
    fn palette_close_cases_not_covered_in_state_tests() {
        assert!(!should_keep_command_palette_open("/done shipped"));
        assert!(!should_keep_command_palette_open("/note "));
    }

    #[test]
    fn command_palette_completion_preserves_suffix_text() {
        assert_eq!(
            complete_command_palette_selection("/now cat", "/now <text>"),
            "/now cat"
        );
        assert_eq!(
            complete_command_palette_selection("   /no cat", "/now <text>"),
            "   /now cat"
        );
        assert_eq!(
            complete_command_palette_selection("/no", "/now <text>"),
            "/now "
        );
    }

    #[test]
    fn ctrl_z_is_treated_as_suspend() {
        assert!(is_suspend_key(crossterm::event::KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
        assert!(!is_suspend_key(crossterm::event::KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
        assert!(!is_suspend_key(crossterm::event::KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
    }
}
