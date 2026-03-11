// Left pane — thread and branch list rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{BranchStatus, ThreadStatus};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::TuiState;
use crate::ui::theme;

/// Render the thread list into the given area.
pub fn render(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut items: Vec<ListItem> = Vec::new();

    for (i, entry) in state.threads.iter().enumerate() {
        let is_selected = i == state.selected_index;
        let is_active = entry.thread.status == ThreadStatus::Active;

        // Thread line
        let marker = if is_active { ">" } else { " " };
        let status_suffix = if entry.thread.status == ThreadStatus::Paused {
            "  paused"
        } else {
            ""
        };

        let style = if is_selected {
            theme::selected()
        } else if is_active {
            theme::active()
        } else {
            theme::text()
        };

        let thread_line = Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::styled(entry.thread.title.clone(), style),
            Span::styled(status_suffix, theme::muted()),
        ]);
        items.push(ListItem::new(thread_line));

        // Branch lines (indented beneath their thread)
        for branch in &entry.branches {
            let branch_style = if branch.status == BranchStatus::Active {
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
                Span::styled("    ", theme::text()),
                Span::styled(branch.title.clone(), branch_style),
                Span::styled(suffix, theme::muted()),
            ]);
            items.push(ListItem::new(branch_line));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(" Threads ", theme::header()));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
