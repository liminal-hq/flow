// Three-pane layout constraint calculations
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::ui::theme;

/// The three regions of the TUI layout.
pub struct AppLayout {
    pub header: Rect,
    pub thread_list: Rect,
    pub reply_pane: Rect,
    pub input_pane: Rect,
}

/// Compute the three-pane layout from the terminal area.
///
/// ```text
/// ┌────────────────────────┬──────────────────────────────┐
/// │ Liminal Flow           │                         flo  │
/// ├────────────────────────┼──────────────────────────────┤
/// │ Thread list (30%)      │ Reply/status pane (70%)      │
/// │                        │                              │
/// ├────────────────────────┴──────────────────────────────┤
/// │ > Input                                               │
/// └───────────────────────────────────────────────────────┘
/// ```
pub fn compute(area: Rect) -> AppLayout {
    // Vertical: header (1) + body (flex) + input (3)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(5),    // body
            Constraint::Length(3), // input
        ])
        .split(area);

    // Horizontal split of the body: 30% thread list, 70% reply
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical[1]);

    AppLayout {
        header: vertical[0],
        thread_list: horizontal[0],
        reply_pane: horizontal[1],
        input_pane: vertical[2],
    }
}

/// Render the header bar.
pub fn render_header(frame: &mut Frame, area: Rect) {
    let width = area.width as usize;
    let left = " Liminal Flow";
    let right = "flo ";
    let padding = width.saturating_sub(left.len() + right.len());

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme::border());

    let header_text = ratatui::text::Line::from(vec![
        Span::styled(left, theme::header()),
        Span::raw(" ".repeat(padding)),
        Span::styled(right, theme::muted()),
    ]);

    let paragraph = ratatui::widgets::Paragraph::new(header_text).block(header_block);
    frame.render_widget(paragraph, area);
}
