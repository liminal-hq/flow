// Terminal UI for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

pub mod app;
pub mod input;
pub mod poll;
pub mod state;
pub mod ui;

use anyhow::Result;
use rusqlite::Connection;

/// Launch the TUI application.
pub async fn run_tui(conn: Connection) -> Result<()> {
    app::run(conn)
}
