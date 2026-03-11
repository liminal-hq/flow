// Handler for `flo branch <text>` — create a branch under the current thread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{Branch, BranchStatus, Capture, CaptureSource, FlowId, Intent};
use liminal_flow_core::rules::normalise_title;
use liminal_flow_context::scope_collector;
use liminal_flow_store::repo::{branch_repo, capture_repo, event_repo, scope_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection, text: &str) -> Result<()> {
    let now = Utc::now();
    let title = normalise_title(text);

    let current_thread = thread_repo::find_active(conn)?;
    let Some(thread) = current_thread else {
        bail!("No active thread. Use `flo now` to start one first.");
    };

    let branch_id = FlowId::new();

    let branch = Branch {
        id: branch_id.clone(),
        thread_id: thread.id.clone(),
        title: title.clone(),
        status: BranchStatus::Active,
        short_summary: None,
        created_at: now,
        updated_at: now,
    };
    branch_repo::upsert(conn, &branch)?;

    let capture = Capture {
        id: FlowId::new(),
        target_type: "branch".into(),
        target_id: branch_id.clone(),
        text: text.to_string(),
        source: CaptureSource::Cli,
        inferred_intent: Some(Intent::StartBranch),
        created_at: now,
    };
    capture_repo::insert(conn, &capture)?;

    // Attach environmental context scopes
    let collected = scope_collector::collect();
    let scopes = scope_collector::as_scopes(&collected, "branch", &branch_id, now);
    for scope in &scopes {
        scope_repo::insert(conn, scope)?;
    }

    let event = AppEvent::BranchStarted {
        branch_id,
        thread_id: thread.id,
        title: title.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Added branch: {title}");
    Ok(())
}
