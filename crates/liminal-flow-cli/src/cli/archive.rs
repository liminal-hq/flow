// Handler for `flo archive` — archive the active focus target
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{BranchStatus, ThreadStatus};
use liminal_flow_store::repo::{branch_repo, event_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection) -> Result<()> {
    let now = Utc::now();

    let Some(thread) = thread_repo::find_active(conn)? else {
        bail!("No active thread or branch to archive.");
    };

    branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;

    if let Some(branch) = branch_repo::find_active_for_thread(conn, &thread.id)? {
        branch_repo::update_status(conn, &branch.id, &BranchStatus::Archived, &now.to_rfc3339())?;

        let event = AppEvent::BranchArchived {
            branch_id: branch.id,
            thread_id: thread.id,
            created_at: now,
        };
        event_repo::insert(conn, &event, "cli")?;

        println!("Archived: {}", branch.title);
        return Ok(());
    }

    thread_repo::update_status(conn, &thread.id, &ThreadStatus::Archived, &now.to_rfc3339())?;

    let event = AppEvent::ThreadArchived {
        thread_id: thread.id,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Archived: {}", thread.title);
    Ok(())
}
