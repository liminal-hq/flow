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

use crate::state::{filtered_slash_commands, TuiState};
use crate::ui::theme;

fn truncate_for_width(text: &str, width: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_string();
    }

    if width <= 1 {
        return "…".repeat(width);
    }

    let visible: String = chars.into_iter().take(width - 1).collect();
    format!("{visible}…")
}

fn command_palette_row(
    cmd: &str,
    desc: &str,
    is_selected: bool,
    popup_width: u16,
) -> Line<'static> {
    let inner_width = usize::from(popup_width.saturating_sub(2));
    let marker = if is_selected { " > " } else { "   " };
    let marker_width = marker.chars().count();
    let gap_width = 1;
    let min_cmd_width = 12;
    let max_cmd_width = 20;
    let available_after_marker = inner_width.saturating_sub(marker_width);
    let cmd_width = available_after_marker
        .saturating_sub(gap_width)
        .min(max_cmd_width)
        .max(min_cmd_width)
        .min(available_after_marker);
    let desc_width = available_after_marker
        .saturating_sub(cmd_width)
        .saturating_sub(gap_width);

    let cmd_text = truncate_for_width(cmd, cmd_width);
    let cmd_padded = format!("{cmd_text:<cmd_width$}");
    let desc_text = truncate_for_width(desc, desc_width);

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

    Line::from(vec![
        Span::styled(marker.to_string(), cmd_style),
        Span::styled(cmd_padded, cmd_style),
        Span::styled(" ".to_string(), desc_style),
        Span::styled(desc_text, desc_style),
    ])
}

/// Render the command palette floating above the input pane.
///
/// The palette anchors to the bottom of the body area (just above the input),
/// growing upward. Each command shows as a row with the command name on the
/// left in accent colour and the description on the right in muted text.
pub fn render(frame: &mut Frame, input_area: Rect, state: &TuiState, query: &str) {
    let filtered = filtered_slash_commands(query);
    let cmd_count = filtered.len().max(1) as u16;
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

    let items: Vec<ListItem> = if filtered.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "   No matching commands",
            theme::muted(),
        )]))]
    } else {
        filtered
            .iter()
            .enumerate()
            .map(|(i, (_command_index, cmd, desc))| {
                let is_selected = i == state.command_palette_index;
                ListItem::new(command_palette_row(
                    cmd,
                    desc,
                    is_selected,
                    popup_area.width,
                ))
            })
            .collect()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::accent())
        .title(Span::styled(" Commands ", theme::header()));

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_row_stays_within_inner_width() {
        let line = command_palette_row(
            "/branch <text>",
            "Start a branch beneath current thread",
            true,
            40,
        );
        let rendered_width: usize = line
            .spans
            .iter()
            .map(|span| span.content.chars().count())
            .sum();
        assert!(rendered_width <= 38);
    }

    #[test]
    fn truncate_for_width_adds_ellipsis_when_needed() {
        assert_eq!(truncate_for_width("branch", 4), "bra…");
        assert_eq!(truncate_for_width("ok", 4), "ok");
    }
}
