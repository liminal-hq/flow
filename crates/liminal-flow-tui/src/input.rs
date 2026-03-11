// Key event dispatch and slash command parsing for the TUI
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use chrono::Utc;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{
    Branch, BranchStatus, Capture, CaptureSource, FlowId, Intent, Thread, ThreadStatus,
};
use liminal_flow_core::rules::{normalise_title, parse_slash_command};
use liminal_flow_store::repo::{branch_repo, capture_repo, event_repo, thread_repo};
use rusqlite::Connection;

/// Result of processing an input line in the TUI.
pub enum InputResult {
    Reply(String),
    Error(String),
    None,
}

/// Process a line of input from the TUI textarea.
/// Parses slash commands or treats plain text as a note.
pub fn process_input(conn: &Connection, raw: &str) -> InputResult {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return InputResult::None;
    }

    // Try to parse as a slash command
    if let Some((intent, arg)) = parse_slash_command(trimmed) {
        return match execute_intent(conn, intent, &arg) {
            Ok(reply) => InputResult::Reply(reply),
            Err(e) => InputResult::Error(e.to_string()),
        };
    }

    // Plain text → treat as a note
    match execute_intent(conn, Intent::AddNote, trimmed) {
        Ok(reply) => InputResult::Reply(reply),
        Err(e) => InputResult::Error(e.to_string()),
    }
}

/// Execute a parsed intent against the database.
fn execute_intent(conn: &Connection, intent: Intent, text: &str) -> Result<String> {
    let now = Utc::now();

    match intent {
        Intent::SetCurrentThread => {
            let title = normalise_title(text);
            let thread_id = FlowId::new();

            if let Some(current) = thread_repo::find_active(conn)? {
                thread_repo::update_status(
                    conn,
                    &current.id,
                    &ThreadStatus::Paused,
                    &now.to_rfc3339(),
                )?;
            }

            let thread = Thread {
                id: thread_id.clone(),
                title: title.clone(),
                raw_origin_text: text.to_string(),
                status: ThreadStatus::Active,
                short_summary: None,
                created_at: now,
                updated_at: now,
            };
            thread_repo::upsert(conn, &thread)?;

            let capture = Capture {
                id: FlowId::new(),
                target_type: "thread".into(),
                target_id: thread_id.clone(),
                text: text.to_string(),
                source: CaptureSource::Keyboard,
                inferred_intent: Some(Intent::SetCurrentThread),
                created_at: now,
            };
            capture_repo::insert(conn, &capture)?;

            let event = AppEvent::ThreadSetCurrent {
                thread_id,
                title: title.clone(),
                raw_text: text.to_string(),
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Current thread: {title}"))
        }

        Intent::StartBranch => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread. Use /now to start one first.");
            };

            let title = normalise_title(text);
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
                source: CaptureSource::Keyboard,
                inferred_intent: Some(Intent::StartBranch),
                created_at: now,
            };
            capture_repo::insert(conn, &capture)?;

            let event = AppEvent::BranchStarted {
                branch_id,
                thread_id: thread.id,
                title: title.clone(),
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Branch started: {title}"))
        }

        Intent::ReturnToParent => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread.");
            };

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
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Returned to parent thread: {}", thread.title))
        }

        Intent::AddNote => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread. Use /now to start one first.");
            };

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
                source: CaptureSource::Keyboard,
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
            event_repo::insert(conn, &event, "tui")?;

            Ok("Note attached.".into())
        }

        Intent::QueryCurrent => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                return Ok("(no active thread)".into());
            };

            let branches = branch_repo::find_by_thread(conn, &thread.id)?;
            let active_count = branches
                .iter()
                .filter(|b| b.status == BranchStatus::Active)
                .count();

            let mut reply = format!("Current thread: {}", thread.title);
            if active_count > 0 {
                reply.push_str(&format!(
                    "\n{} active branch{}",
                    active_count,
                    if active_count == 1 { "" } else { "es" }
                ));
            }
            Ok(reply)
        }

        Intent::Pause => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread to pause.");
            };

            thread_repo::update_status(
                conn,
                &thread.id,
                &ThreadStatus::Paused,
                &now.to_rfc3339(),
            )?;

            let event = AppEvent::ThreadPaused {
                thread_id: thread.id,
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Paused thread: {}", thread.title))
        }

        Intent::Ambiguous => {
            // Treat ambiguous input as a note
            return execute_intent(conn, Intent::AddNote, text);
        }

        Intent::Done => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread to mark done.");
            };

            thread_repo::update_status(
                conn,
                &thread.id,
                &ThreadStatus::Done,
                &now.to_rfc3339(),
            )?;

            let event = AppEvent::ThreadMarkedDone {
                thread_id: thread.id,
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Done: {}", thread.title))
        }
    }
}
