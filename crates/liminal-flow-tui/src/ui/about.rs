// About overlay — app info, logo, and credits
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
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

/// Brand gradient colours from the Liminal HQ palette.
const ORANGE: Color = Color::Rgb(0xff, 0xaa, 0x40);
const PURPLE: Color = Color::Rgb(0xa7, 0x8b, 0xfa);

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    // Logo — each line gets a gradient colour stepping orange → purple
    let logo_colours = [
        Color::Rgb(0xff, 0xaa, 0x40), // orange
        Color::Rgb(0xf4, 0x7f, 0x5e), // orange-pink
        Color::Rgb(0xd0, 0x64, 0x8c), // pink
        Color::Rgb(0xb0, 0x78, 0xc8), // pink-purple
        Color::Rgb(0xa7, 0x8b, 0xfa), // purple
    ];
    for (i, logo_line) in LOGO.iter().enumerate() {
        let colour = logo_colours[i % logo_colours.len()];
        lines.push(Line::from(Span::styled(
            *logo_line,
            Style::default().fg(colour),
        )));
    }

    lines.push(Line::from(""));

    // App name (brand colours) + version
    lines.push(Line::from(vec![
        Span::styled(
            "  Liminal",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            "Flow",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  v{VERSION}"), theme::muted()),
    ]));

    // Tagline
    lines.push(Line::from(Span::styled(
        "  Terminal-native working memory for developers",
        theme::text(),
    )));

    lines.push(Line::from(""));

    // Studio + copyright
    lines.push(Line::from(vec![
        Span::styled("  Liminal", Style::default().fg(ORANGE)),
        Span::styled(" HQ", Style::default().fg(PURPLE)),
    ]));
    lines.push(Line::from(Span::styled(
        "  (c) 2026 Liminal HQ, Scott Morris",
        theme::muted(),
    )));
    lines.push(Line::from(Span::styled("  MIT Licence", theme::muted())));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE))
        .title(Span::styled(
            " About ",
            Style::default()
                .fg(ORANGE)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup_area);
}
