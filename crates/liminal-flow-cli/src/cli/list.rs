// Handler for `flo list` — list active and paused threads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use liminal_flow_core::model::ThreadStatus;
use liminal_flow_store::repo::thread_repo;
use rusqlite::Connection;

pub fn handle(conn: &Connection) -> Result<()> {
    let threads =
        thread_repo::list_by_statuses(conn, &[ThreadStatus::Active, ThreadStatus::Paused])?;

    if threads.is_empty() {
        println!("No active threads.");
        return Ok(());
    }

    for thread in &threads {
        let marker = if thread.status == ThreadStatus::Active {
            ">"
        } else {
            " "
        };
        let status_label = if thread.status == ThreadStatus::Paused {
            "  paused"
        } else {
            ""
        };
        println!("{marker} {}{status_label}", thread.title);
    }

    Ok(())
}
