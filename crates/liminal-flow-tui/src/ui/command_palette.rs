// Command palette popup — shown when `/` is typed on an empty input line
//
// Renders a floating list of slash commands above the input pane,
// inspired by Claude Code and Codex command menus.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem};
use ratatui::Frame;

use crate::state::{TuiState, SLASH_COMMANDS};
use crate::ui::theme;

/// Render the command palette floating above the input pane.
///
/// The palette anchors to the bottom of the body area (just above the input),
/// growing upward. Each command shows as a row with the command name on the
/// left in accent colour and the description on the right in muted text.
pub fn render(frame: &mut Frame, input_area: Rect, state: &TuiState) {
    let cmd_count = SLASH_COMMANDS.len() as u16;
    // Height = commands + 2 for borders
    let popup_height = (cmd_count + 2).min(input_area.y);
    let popup_width = input_area.width.min(60);

    // Position just above the input pane, left-aligned with it
    let popup_area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width,
        height: popup_height,
    };

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = SLASH_COMMANDS
        .iter()
        .enumerate()
        .map(|(i, (cmd, desc))| {
            let is_selected = i == state.command_palette_index;
            let cmd_style = if is_selected {
                theme::selected()
            } else {
                theme::accent()
            };
            let desc_style = if is_selected {
                theme::text()
            } else {
                theme::muted()
            };
            let marker = if is_selected { " > " } else { "   " };

            ListItem::new(Line::from(vec![
                Span::styled(marker, cmd_style),
                Span::styled(format!("{cmd:<20}"), cmd_style),
                Span::styled(*desc, desc_style),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::accent())
        .title(Span::styled(" Commands ", theme::header()));

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);
}
