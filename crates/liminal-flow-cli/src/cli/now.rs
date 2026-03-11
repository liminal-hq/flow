// Handler for `flo now <text>` — set or replace the current thread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{Capture, CaptureSource, FlowId, Intent, ThreadStatus};
use liminal_flow_core::rules::normalise_title;
use liminal_flow_context::scope_collector;
use liminal_flow_store::repo::{capture_repo, event_repo, scope_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection, text: &str) -> Result<()> {
    let now = Utc::now();
    let title = normalise_title(text);
    let thread_id = FlowId::new();

    // Pause any currently active thread
    if let Some(current) = thread_repo::find_active(conn)? {
        thread_repo::update_status(conn, &current.id, &ThreadStatus::Paused, &now.to_rfc3339())?;
    }

    // Create the new thread via upsert
    let thread = liminal_flow_core::model::Thread {
        id: thread_id.clone(),
        title: title.clone(),
        raw_origin_text: text.to_string(),
        status: ThreadStatus::Active,
        short_summary: None,
        created_at: now,
        updated_at: now,
    };
    thread_repo::upsert(conn, &thread)?;

    // Store the capture
    let capture = Capture {
        id: FlowId::new(),
        target_type: "thread".into(),
        target_id: thread_id.clone(),
        text: text.to_string(),
        source: CaptureSource::Cli,
        inferred_intent: Some(Intent::SetCurrentThread),
        created_at: now,
    };
    capture_repo::insert(conn, &capture)?;

    // Attach environmental context scopes
    let collected = scope_collector::collect();
    let scopes = scope_collector::as_scopes(&collected, "thread", &thread_id, now);
    for scope in &scopes {
        scope_repo::insert(conn, scope)?;
    }

    // Record the event
    let event = AppEvent::ThreadSetCurrent {
        thread_id,
        title: title.clone(),
        raw_text: text.to_string(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Current thread: {title}");
    Ok(())
}
