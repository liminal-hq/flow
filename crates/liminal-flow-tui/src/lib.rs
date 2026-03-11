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
pub async fn run_tui(_conn: Connection) -> Result<()> {
    // Placeholder — will be implemented in Phase 4.
    println!("Liminal Flow TUI — coming soon.");
    Ok(())
}
