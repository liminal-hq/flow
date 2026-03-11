// Left pane — thread and branch list rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{BranchStatus, ThreadStatus};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::{SelectedItem, TuiState};
use crate::ui::theme;

/// Render the thread list into the given area.
pub fn render(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut items: Vec<ListItem> = Vec::new();

    for (i, entry) in state.threads.iter().enumerate() {
        let is_thread_selected = state.selected == SelectedItem::Thread(i);
        let is_active = entry.thread.status == ThreadStatus::Active;
        let is_expanded = state.is_expanded(i);
        let has_branches = !entry.branches.is_empty();

        // Thread line with expand/collapse indicator
        let marker = if is_active { ">" } else { " " };
        let expand_indicator = if has_branches {
            if is_expanded {
                "▼ "
            } else {
                "▶ "
            }
        } else {
            "  "
        };
        let status_suffix = if entry.thread.status == ThreadStatus::Paused {
            "  paused"
        } else {
            ""
        };

        let style = if is_thread_selected {
            theme::selected()
        } else if is_active {
            theme::active()
        } else {
            theme::text()
        };

        let thread_line = Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::styled(expand_indicator, theme::muted()),
            Span::styled(entry.thread.title.clone(), style),
            Span::styled(status_suffix, theme::muted()),
        ]);
        items.push(ListItem::new(thread_line));

        // Branch lines (indented beneath their thread) — only when expanded
        if is_expanded {
            for (j, branch) in entry.branches.iter().enumerate() {
                let is_branch_selected = state.selected == SelectedItem::Branch(i, j);

                let branch_style = if is_branch_selected {
                    theme::selected()
                } else if branch.status == BranchStatus::Active {
                    theme::accent()
                } else {
                    theme::muted()
                };

                let suffix = if branch.status != BranchStatus::Active {
                    format!("  ({})", branch.status)
                } else {
                    String::new()
                };

                let branch_line = Line::from(vec![
                    Span::styled("      ", theme::text()),
                    Span::styled(branch.title.clone(), branch_style),
                    Span::styled(suffix, theme::muted()),
                ]);
                items.push(ListItem::new(branch_line));
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(" Threads ", theme::header()));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
