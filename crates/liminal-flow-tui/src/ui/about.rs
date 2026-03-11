// About overlay — app info, logo, and credits
//
// Shown when the user presses `?` from Normal mode (the help overlay)
// or via a dedicated `/about` in future. For now, integrated into the
// shortcut hints as a fun branded touch.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::theme;

/// ASCII art logo for Liminal Flow — a stylised wave/flow motif.
const LOGO: &[&str] = &[
    r"        __ _            ",
    r"    ___/ _| |___      __",
    r"   / __| |_| / _ \    \ \",
    r"  | _||  _| | (_) |    > >",
    r"   \_| |_| |_|\___/   /_/",
];

const APP_NAME: &str = "Liminal Flow";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const TAGLINE: &str = "Terminal-native working memory for developers";
const STUDIO: &str = "Liminal HQ";
const COPYRIGHT: &str = "(c) 2026 Liminal HQ, Scott Morris";
const LICENCE: &str = "MIT Licence";

/// Render the about overlay centred on screen.
pub fn render(frame: &mut Frame, area: Rect) {
    let popup_width = 52.min(area.width.saturating_sub(4));
    let popup_height = 16.min(area.height.saturating_sub(2));

    let vert = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area);
    let horiz = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(vert[0]);
    let popup_area = horiz[0];

    frame.render_widget(Clear, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // Logo
    for logo_line in LOGO {
        lines.push(Line::from(Span::styled(*logo_line, theme::accent())));
    }

    lines.push(Line::from(""));

    // App name + version
    lines.push(Line::from(vec![
        Span::styled(format!("  {APP_NAME}"), theme::active()),
        Span::styled(format!("  v{VERSION}"), theme::muted()),
    ]));

    // Tagline
    lines.push(Line::from(Span::styled(
        format!("  {TAGLINE}"),
        theme::text(),
    )));

    lines.push(Line::from(""));

    // Studio + copyright
    lines.push(Line::from(Span::styled(
        format!("  {STUDIO}"),
        theme::header(),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {COPYRIGHT}"),
        theme::muted(),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {LICENCE}"),
        theme::muted(),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .title(Span::styled(" About ", theme::header()));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup_area);
}
