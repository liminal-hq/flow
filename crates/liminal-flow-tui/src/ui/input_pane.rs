// Bottom pane — chat input with tui-textarea
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_textarea::TextArea;

use crate::state::Mode;
use crate::ui::theme;

/// Render the input textarea into the given area.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    textarea: &TextArea,
    mode: Mode,
    active_target: Option<&str>,
) {
    let mode_label: String = match (mode, active_target) {
        (Mode::Insert, Some(target)) => format!(" > Capture ({target}) "),
        (Mode::Insert, None) => " > Capture (no active target) ".into(),
        (Mode::Normal, _) => " Normal ".into(),
        (Mode::Help | Mode::About, _) => " Help ".into(),
    };

    let border_style = if mode == Mode::Insert {
        theme::accent()
    } else {
        theme::border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(mode_label, theme::header()));

    // We need to clone the block into the textarea for rendering
    // tui-textarea renders itself with its own block
    let mut ta = textarea.clone();
    ta.set_block(block);
    ta.set_cursor_line_style(ratatui::style::Style::default());

    frame.render_widget(&ta, area);
}
