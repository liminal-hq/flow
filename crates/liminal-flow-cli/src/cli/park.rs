// Handler for `flo park` — park the active branch
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

    let Some(thread) = thread_repo::find_active(conn)? else {
        bail!("No active thread.");
    };

    branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;
    let Some(branch) = branch_repo::find_active_for_thread(conn, &thread.id)? else {
        bail!("No active branch to park.");
    };

    branch_repo::update_status(conn, &branch.id, &BranchStatus::Parked, &now.to_rfc3339())?;

    let event = AppEvent::BranchParked {
        branch_id: branch.id,
        thread_id: thread.id,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Parked branch: {}", branch.title);
    Ok(())
}
