// Shortcut hints bar — shown when `?` is typed on an empty input line
//
// Renders a compact hints area below the input box showing available
// keyboard shortcuts, inspired by Claude Code and Codex help displays.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::state::SHORTCUT_HINTS;
use crate::ui::theme;

/// Render the shortcut hints floating above the input pane.
///
/// Displays a compact multi-column hint block similar to Claude Code's
/// `?` shortcut display.
pub fn render(frame: &mut Frame, input_area: Rect) {
    let row_count = SHORTCUT_HINTS.len() as u16;
    let popup_height = row_count + 1; // +1 for a top border line
    let popup_width = input_area.width;

    let popup_area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // Separator line
    let sep = "─".repeat(popup_width.saturating_sub(1) as usize);
    lines.push(Line::from(Span::styled(sep, theme::border())));

    let col_width = popup_width as usize / 2;

    for (left, right) in SHORTCUT_HINTS {
        let left_padded = format!("  {left:<width$}", width = col_width.saturating_sub(2));
        let mut spans = vec![Span::styled(left_padded, theme::muted())];
        if !right.is_empty() {
            spans.push(Span::styled((*right).to_string(), theme::muted()));
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, popup_area);
}
