// Handler for `flo pause` — pause the current thread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::ThreadStatus;
use liminal_flow_store::repo::{event_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection) -> Result<()> {
    let now = Utc::now();

    let current_thread = thread_repo::find_active(conn)?;
    let Some(thread) = current_thread else {
        bail!("No active thread to pause.");
    };

    thread_repo::update_status(conn, &thread.id, &ThreadStatus::Paused, &now.to_rfc3339())?;

    let event = AppEvent::ThreadPaused {
        thread_id: thread.id,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Paused thread: {}", thread.title);
    Ok(())
}
