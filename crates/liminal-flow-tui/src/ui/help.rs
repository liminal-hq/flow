// Help overlay rendering
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme;

/// Section header marker — rendered differently from command rows.
const S: &str = "";

const HELP_TEXT: &[(&str, &str)] = &[
    (S, "Slash Commands"),
    ("/now /branch /back", "Set thread, branch, or return"),
    ("/note /where", "Add a note or show current state"),
    ("/pause /done", "Pause or finish current thread"),
    (S, "Insert Mode"),
    ("/ (empty line)", "Open command palette"),
    ("? (empty line)", "Show shortcut hints"),
    ("Up / Down", "Navigate threads & branches"),
    ("Enter (empty)", "Expand/collapse branches"),
    ("Enter (text)", "Submit input"),
    ("PageUp / PageDown", "Scroll the Status pane"),
    ("Esc", "Switch to Normal mode"),
    (S, "Normal Mode"),
    ("i", "Switch to Insert mode"),
    ("j / k / Up / Down", "Navigate threads & branches"),
    ("Enter", "Expand/collapse branches"),
    ("r", "Resume selected item"),
    ("p", "Park selected branch"),
    ("PageUp / PageDown", "Scroll the Status pane"),
    ("?", "Toggle this help"),
    ("a", "About"),
    ("q", "Quit"),
    (S, "Help Mode"),
    ("j / k / Up / Down", "Scroll the Help content"),
    ("PageUp / PageDown", "Scroll faster"),
    ("Esc / ? / q", "Close Help"),
];

/// Render the help overlay centred on screen.
pub fn render(frame: &mut Frame, area: Rect, scroll: u16) {
    let popup_width = 64.min(area.width.saturating_sub(4));
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
            if *cmd == S {
                // Section header
                Line::from(Span::styled(
                    format!(" {desc}"),
                    theme::header().add_modifier(Modifier::BOLD),
                ))
            } else if cmd.is_empty() {
                Line::from("")
            } else {
                Line::from(vec![
                    Span::styled(format!("  {cmd:<22}"), theme::accent()),
                    Span::styled(*desc, theme::text()),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::accent())
        .title(Span::styled(" Help ", theme::header()));

    let paragraph = Paragraph::new(lines).block(block).scroll((scroll, 0));
    frame.render_widget(paragraph, popup_area);
}
