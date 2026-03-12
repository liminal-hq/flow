// Colour theme constants for Liminal Flow TUI
//
// Adapted from smdu's default theme. Does not force a background colour —
// uses the terminal's default.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::style::{Color, Modifier, Style};

// ── Palette ──────────────────────────────────────────────────────────

pub const TEXT: Color = Color::Rgb(0xd2, 0xd8, 0xe1);
pub const MUTED: Color = Color::Rgb(0x7c, 0x87, 0x96);
pub const ACCENT: Color = Color::Rgb(0x5a, 0xa2, 0xff);
pub const ACTIVE: Color = Color::Rgb(0x2e, 0xc6, 0x6a);
pub const LINE: Color = Color::Rgb(0x2e, 0x35, 0x40);
pub const HEADER: Color = Color::Rgb(0x9a, 0xa4, 0xb2);
pub const ERROR: Color = Color::Rgb(0xfc, 0xa5, 0xa5);
pub const DONE: Color = Color::Rgb(0xe6, 0xb4, 0x50);

// ── Compound styles ─────────────────────────────────────────────────

pub fn text() -> Style {
    Style::default().fg(TEXT)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn accent() -> Style {
    Style::default().fg(ACCENT)
}

pub fn active() -> Style {
    Style::default().fg(ACTIVE)
}

pub fn header() -> Style {
    Style::default().fg(HEADER)
}

pub fn error() -> Style {
    Style::default().fg(ERROR)
}

pub fn done() -> Style {
    Style::default().fg(DONE)
}

pub fn selected() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::default().fg(LINE)
}
