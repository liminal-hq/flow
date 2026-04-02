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
    ("/now /branch", "Set thread or start branch"),
    ("/back", "Return from active branch to parent"),
    ("/park", "Park the selected branch"),
    ("/archive", "Archive the selected item"),
    ("/note /where", "Note selected item or show current state"),
    ("/resume", "Resume the selected item"),
    (
        "/pause /done",
        "Pause selected thread or finish selected item",
    ),
    (S, "Insert Mode"),
    ("/ (empty line)", "Open command palette"),
    ("? (empty line)", "Show shortcut hints"),
    ("Up / Down", "Navigate threads & branches"),
    ("Ctrl+Up / Ctrl+Down", "Browse session note history"),
    ("Enter (empty)", "Expand/collapse branches"),
    ("Ctrl+J", "Insert newline"),
    ("Mouse wheel", "Scroll Threads or Status"),
    ("Enter (text)", "Submit input"),
    ("PageUp / PageDown", "Scroll the Status pane"),
    ("Ctrl+Z", "Suspend flo and return to shell"),
    ("Esc", "Switch to Normal mode"),
    (S, "Normal Mode"),
    ("i", "Switch to Insert mode"),
    ("j / k / Up / Down", "Navigate threads & branches"),
    ("Enter", "Expand/collapse branches"),
    ("r", "Resume selected item"),
    ("p", "Park selected branch"),
    ("d", "Mark selected item done"),
    ("A", "Archive selected item"),
    ("PageUp / PageDown", "Scroll the Status pane"),
    ("Mouse wheel", "Scroll hovered pane"),
    ("Ctrl+Z", "Suspend flo and return to shell"),
    ("?", "Toggle this help"),
    ("a", "About"),
    ("q", "Quit"),
    (S, "Help Mode"),
    ("j / k / Up / Down", "Scroll the Help content"),
    ("PageUp / PageDown", "Scroll faster"),
    ("Mouse wheel", "Scroll Help under pointer"),
    ("Esc / ? / q", "Close Help"),
];

/// Render the help overlay centred on screen.
pub fn popup_area(area: Rect) -> Rect {
    let popup_width = 64.min(area.width.saturating_sub(4));
    let popup_height = (HELP_TEXT.len() as u16 + 4).min(area.height.saturating_sub(2));

    let vert = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area);
    let horiz = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(vert[0]);
    horiz[0]
}

/// Render the help overlay centred on screen.
pub fn render(frame: &mut Frame, area: Rect, scroll: u16) {
    let popup_area = popup_area(area);

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
