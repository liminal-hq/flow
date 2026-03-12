// Right pane — reply and status voice rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::TuiState;
use crate::ui::theme;

/// Render the reply/status pane into the given area.
pub fn render(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(title) = state.selected_title() {
        lines.push(Line::from(vec![
            Span::styled(format!("{}: ", state.selected_kind_label()), theme::muted()),
            Span::styled(
                title,
                if state.selected_is_active() {
                    theme::active()
                } else {
                    theme::selected()
                },
            ),
        ]));

        if let Some(parent_title) = state.selected_parent_title() {
            lines.push(Line::from(vec![
                Span::styled("Thread: ", theme::muted()),
                Span::styled(parent_title, theme::text()),
            ]));
        }

        if let Some(status) = state.selected_status_label() {
            lines.push(Line::from(Span::styled(
                format!("Status: {status}"),
                theme::muted(),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "(no active thread)",
            theme::muted(),
        )));
    }

    // Recent notes
    if !state.selected_notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Notes", theme::accent())));
        for (index, note) in state.selected_notes.iter().enumerate() {
            if index > 0 {
                lines.push(Line::from(Span::styled("  ---", theme::border())));
            }

            lines.push(Line::from(vec![
                Span::styled("  ", theme::text()),
                Span::styled(
                    note.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    theme::muted(),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ", theme::text()),
                Span::styled(note.text.clone(), theme::text()),
            ]));
        }
    }

    // Last reply or error
    if let Some(ref err) = state.error_message {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(err.clone(), theme::error())));
    } else if let Some(ref reply) = state.last_reply {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(reply.clone(), theme::text())));
    }

    let has_scope = state.selected_scope_context.repo.is_some()
        || state.selected_scope_context.git_branch.is_some()
        || state.selected_scope_context.cwd.is_some();
    if has_scope {
        lines.push(Line::from(""));
        if let Some(ref repo) = state.selected_scope_context.repo {
            lines.push(Line::from(vec![
                Span::styled("Repo: ", theme::muted()),
                Span::styled(repo.clone(), theme::text()),
            ]));
        }
        if let Some(ref git_branch) = state.selected_scope_context.git_branch {
            lines.push(Line::from(vec![
                Span::styled("Git: ", theme::muted()),
                Span::styled(git_branch.clone(), theme::text()),
            ]));
        }
        if let Some(ref cwd) = state.selected_scope_context.cwd {
            lines.push(Line::from(vec![
                Span::styled("Dir: ", theme::muted()),
                Span::styled(cwd.clone(), theme::text()),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border())
        .title(Span::styled(" Status ", theme::header()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((state.status_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
