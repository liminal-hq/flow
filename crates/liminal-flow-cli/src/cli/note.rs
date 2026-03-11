// Handler for `flo note <text>` — attach a note to the current focus target
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{Capture, CaptureSource, FlowId, Intent};
use liminal_flow_store::repo::{branch_repo, capture_repo, event_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection, text: &str) -> Result<()> {
    let now = Utc::now();

    let current_thread = thread_repo::find_active(conn)?;
    let Some(thread) = current_thread else {
        bail!("No active thread. Use `flo now` to start one first.");
    };

    // Attach to the active branch if one exists, otherwise to the thread
    let (target_type, target_id) =
        if let Some(branch) = branch_repo::find_active_for_thread(conn, &thread.id)? {
            ("branch".to_string(), branch.id)
        } else {
            ("thread".to_string(), thread.id.clone())
        };

    let capture_id = FlowId::new();
    let capture = Capture {
        id: capture_id.clone(),
        target_type: target_type.clone(),
        target_id: target_id.clone(),
        text: text.to_string(),
        source: CaptureSource::Cli,
        inferred_intent: Some(Intent::AddNote),
        created_at: now,
    };
    capture_repo::insert(conn, &capture)?;

    let event = AppEvent::NoteAttached {
        capture_id,
        target_type,
        target_id,
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Note attached.");
    Ok(())
}
