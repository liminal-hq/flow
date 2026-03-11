// ANSI colour helpers for CLI output
//
// Uses the same palette as the TUI theme for visual consistency.
// Respects NO_COLOR environment variable.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::env;
use std::io::{self, IsTerminal};

/// Whether colour output is enabled (terminal + no NO_COLOR).
pub fn enabled() -> bool {
    io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none()
}

// ── ANSI helpers ──────────────────────────────────────────

pub fn green(s: &str) -> String {
    if enabled() {
        format!("\x1b[38;2;46;198;106m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn blue(s: &str) -> String {
    if enabled() {
        format!("\x1b[38;2;90;162;255m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn muted(s: &str) -> String {
    if enabled() {
        format!("\x1b[38;2;124;135;150m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn orange(s: &str) -> String {
    if enabled() {
        format!("\x1b[38;2;255;170;64m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    if enabled() {
        format!("\x1b[1m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}
