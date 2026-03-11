// Right pane — reply and status voice rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::TuiState;
use crate::ui::theme;

/// Render the reply/status pane into the given area.
pub fn render(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines: Vec<Line> = Vec::new();

    // Current thread info
    if let Some(active) = state.active_thread() {
        lines.push(Line::from(vec![
            Span::styled("Current thread: ", theme::muted()),
            Span::styled(active.thread.title.clone(), theme::active()),
        ]));

        let active_branches: Vec<_> = active
            .branches
            .iter()
            .filter(|b| b.status == liminal_flow_core::model::BranchStatus::Active)
            .collect();

        if !active_branches.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(
                    "{} active branch{}",
                    active_branches.len(),
                    if active_branches.len() == 1 { "" } else { "es" }
                ),
                theme::muted(),
            )));
        }

        lines.push(Line::from(""));

        // Scope context
        if let Some(ref repo) = state.scope_context.repo {
            lines.push(Line::from(vec![
                Span::styled("Repo: ", theme::muted()),
                Span::styled(repo.clone(), theme::text()),
            ]));
        }
        if let Some(ref git_branch) = state.scope_context.git_branch {
            lines.push(Line::from(vec![
                Span::styled("Git: ", theme::muted()),
                Span::styled(git_branch.clone(), theme::text()),
            ]));
        }
        if let Some(ref cwd) = state.scope_context.cwd {
            lines.push(Line::from(vec![
                Span::styled("Dir: ", theme::muted()),
                Span::styled(cwd.clone(), theme::text()),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "(no active thread)",
            theme::muted(),
        )));
    }

    // Recent notes
    if !state.recent_notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Notes", theme::header())));
        for note in &state.recent_notes {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(" Status ", theme::header()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
