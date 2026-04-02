// Three-pane layout constraint calculations
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::ui::theme;

const MIN_INPUT_PANE_HEIGHT: u16 = 3;
const MAX_INPUT_PANE_HEIGHT: u16 = 5;

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
pub fn input_pane_height(line_count: usize) -> u16 {
    let content_height = u16::try_from(line_count)
        .unwrap_or(u16::MAX)
        .saturating_add(2);
    content_height.clamp(MIN_INPUT_PANE_HEIGHT, MAX_INPUT_PANE_HEIGHT)
}

pub fn compute(area: Rect, input_pane_height: u16) -> AppLayout {
    // Vertical: header (1) + body (flex) + input (dynamic)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                 // header
            Constraint::Min(5),                    // body
            Constraint::Length(input_pane_height), // input
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

/// Render the header bar with colourful branding.
pub fn render_header(frame: &mut Frame, area: Rect) {
    use ratatui::style::{Color, Modifier, Style};

    let width = area.width as usize;
    // "< flo >" is 9 chars + trailing space = 10
    let right_len = 10;
    let left_parts_len = 15; // " Liminal Flow "
    let padding = width.saturating_sub(left_parts_len + right_len);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme::border());

    // Brand colours from the Liminal HQ palette
    let orange = Color::Rgb(0xff, 0xaa, 0x40);
    let purple = Color::Rgb(0xa7, 0x8b, 0xfa);

    let header_text = ratatui::text::Line::from(vec![
        Span::styled(" Liminal ", theme::header()),
        Span::styled(
            "Flow",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(" ".repeat(padding)),
        // Colourful <flo> prompt
        Span::styled("<", Style::default().fg(orange)),
        Span::styled(
            " flo ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(">", Style::default().fg(purple)),
        Span::raw(" "),
    ]);

    let paragraph = ratatui::widgets::Paragraph::new(header_text).block(header_block);
    frame.render_widget(paragraph, area);
}

/// Return whether a terminal column/row point lies within a rectangle.
pub fn contains_point(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_pane_height_stays_compact_for_single_line_input() {
        assert_eq!(input_pane_height(1), MIN_INPUT_PANE_HEIGHT);
    }

    #[test]
    fn input_pane_height_grows_for_multiline_input() {
        assert_eq!(input_pane_height(2), 4);
        assert_eq!(input_pane_height(3), MAX_INPUT_PANE_HEIGHT);
    }

    #[test]
    fn input_pane_height_stops_growing_after_cap() {
        assert_eq!(input_pane_height(10), MAX_INPUT_PANE_HEIGHT);
    }

    #[test]
    fn compute_uses_requested_input_pane_height() {
        let layout = compute(Rect::new(0, 0, 120, 40), 4);
        assert_eq!(layout.input_pane.height, 4);
    }
}
