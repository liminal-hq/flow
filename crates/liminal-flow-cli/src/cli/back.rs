// Handler for `flo back` — return to the parent thread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::BranchStatus;
use liminal_flow_store::repo::{branch_repo, event_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection) -> Result<()> {
    let now = Utc::now();

    let current_thread = thread_repo::find_active(conn)?;
    let Some(thread) = current_thread else {
        bail!("No active thread.");
    };

    // Find all active branches for this thread and park them
    let branches = branch_repo::find_by_thread(conn, &thread.id)?;
    let mut parked_ids = Vec::new();

    for branch in &branches {
        if branch.status == BranchStatus::Active {
            branch_repo::update_status(
                conn,
                &branch.id,
                &BranchStatus::Parked,
                &now.to_rfc3339(),
            )?;
            parked_ids.push(branch.id.clone());
        }
    }

    let event = AppEvent::ReturnedToParent {
        thread_id: thread.id,
        parked_branch_ids: parked_ids,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Returned to parent thread: {}", thread.title);
    Ok(())
}
