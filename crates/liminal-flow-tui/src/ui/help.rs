// Help overlay rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::theme;

const HELP_TEXT: &[(&str, &str)] = &[
    ("/now <text>", "Set or replace the current thread"),
    (
        "/branch <text>",
        "Create a branch beneath the current thread",
    ),
    ("/back", "Return to the parent thread"),
    ("/note <text>", "Attach a note (or just type plain text)"),
    ("/where", "Show current thread and branches"),
    ("/pause", "Pause the current thread"),
    ("/done", "Mark the current thread done"),
    ("", ""),
    ("/ (empty line)", "Open command palette"),
    ("? (empty line)", "Show shortcut hints"),
    ("Up / Down", "Navigate thread list"),
    ("Enter (empty)", "Expand/collapse thread branches"),
    ("Enter (text)", "Submit input"),
    ("Esc", "Switch to Normal mode"),
    ("i", "Switch to Insert mode (from Normal)"),
    ("?", "Toggle help (from Normal mode)"),
    ("r", "Resume selected thread (Normal mode)"),
    ("a", "About (from Normal mode)"),
    ("q", "Quit (from Normal mode)"),
    ("j / k", "Navigate thread list (Normal mode)"),
];

/// Render the help overlay centred on screen.
pub fn render(frame: &mut Frame, area: Rect) {
    // Centre a box that is 60 wide and 18 tall (or as much as fits)
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = (HELP_TEXT.len() as u16 + 4).min(area.height.saturating_sub(2));

    let vert = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area);
    let horiz = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(vert[0]);
    let popup_area = horiz[0];

    frame.render_widget(Clear, popup_area);

    let lines: Vec<Line> = HELP_TEXT
        .iter()
        .map(|(cmd, desc)| {
            if cmd.is_empty() {
                Line::from("")
            } else {
                Line::from(vec![
                    Span::styled(format!("{cmd:<20}"), theme::accent()),
                    Span::styled(*desc, theme::text()),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .title(Span::styled(" Help ", theme::header()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup_area);
}
