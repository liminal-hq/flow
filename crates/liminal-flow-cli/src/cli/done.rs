// Handler for `flo done` — mark the active focus target done
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

    let current_thread = thread_repo::find_active(conn)?;
    let Some(thread) = current_thread else {
        bail!("No active thread or branch to mark done.");
    };

    branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;

    if let Some(branch) = branch_repo::find_active_for_thread(conn, &thread.id)? {
        branch_repo::update_status(conn, &branch.id, &BranchStatus::Done, &now.to_rfc3339())?;

        let event = AppEvent::BranchMarkedDone {
            branch_id: branch.id,
            thread_id: thread.id,
            created_at: now,
        };
        event_repo::insert(conn, &event, "cli")?;

        println!("Done: {}", branch.title);
        return Ok(());
    }

    thread_repo::update_status(conn, &thread.id, &ThreadStatus::Done, &now.to_rfc3339())?;

    let event = AppEvent::ThreadMarkedDone {
        thread_id: thread.id,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Done: {}", thread.title);
    Ok(())
}
