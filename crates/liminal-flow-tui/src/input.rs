// Key event dispatch and slash command parsing for the TUI
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use chrono::{DateTime, Utc};
use liminal_flow_context::scope_collector;
use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::{
    Branch, BranchStatus, Capture, CaptureSource, FlowId, Intent, Thread, ThreadStatus,
};
use liminal_flow_core::rules::{normalise_title, parse_slash_command};
use liminal_flow_store::repo::{branch_repo, capture_repo, event_repo, scope_repo, thread_repo};
use rusqlite::Connection;

/// Result of processing an input line in the TUI.
pub enum InputResult {
    Reply(String),
    Error(String),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTarget {
    Thread(FlowId),
    Branch {
        thread_id: FlowId,
        branch_id: FlowId,
    },
}

/// Parse the user input into an intent when it maps cleanly to a known command.
pub fn parsed_intent(raw: &str) -> Option<Intent> {
    parse_slash_command(raw.trim()).map(|(intent, _)| intent)
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

    if trimmed.starts_with('/') {
        return InputResult::Error(format!("Unknown command: {trimmed}"));
    }

    // Plain text → treat as a note
    match execute_intent(conn, Intent::AddNote, trimmed) {
        Ok(reply) => InputResult::Reply(reply),
        Err(e) => InputResult::Error(e.to_string()),
    }
}

/// Process input with access to the current TUI selection.
pub fn process_input_with_target(
    conn: &Connection,
    raw: &str,
    target: Option<&CommandTarget>,
) -> InputResult {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return InputResult::None;
    }

    if let Some((intent, arg)) = parse_slash_command(trimmed) {
        return match execute_intent_with_target(conn, intent, &arg, target) {
            Ok(reply) => InputResult::Reply(reply),
            Err(e) => InputResult::Error(e.to_string()),
        };
    }

    if trimmed.starts_with('/') {
        return InputResult::Error(format!("Unknown command: {trimmed}"));
    }

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

            // Attach environmental context scopes
            let collected = scope_collector::collect();
            let scopes = scope_collector::as_scopes(&collected, "thread", &thread_id, now);
            for scope in &scopes {
                scope_repo::insert(conn, scope)?;
            }

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

            let existing_branches = branch_repo::find_by_thread(conn, &thread.id)?;
            for existing in &existing_branches {
                if existing.status == BranchStatus::Active {
                    branch_repo::update_status(
                        conn,
                        &existing.id,
                        &BranchStatus::Parked,
                        &now.to_rfc3339(),
                    )?;
                }
            }

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
                thread_id: thread.id.clone(),
                parked_branch_ids: parked_ids.clone(),
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            if !text.trim().is_empty() {
                if let Some(branch_id) = parked_ids.first() {
                    attach_note_to_target(conn, "branch", branch_id, text)?;
                } else {
                    attach_note_to_target(conn, "thread", &thread.id, text)?;
                }
            }

            Ok(format!("Returned to parent thread: {}", thread.title))
        }

        Intent::AddNote => {
            let _ = thread_repo::normalize_active(conn, &now.to_rfc3339())?;
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread. Use /now to start one first.");
            };

            let _ = branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;
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

            if !text.trim().is_empty() {
                attach_note_to_target(conn, "thread", &thread.id, text)?;
            }

            thread_repo::update_status(conn, &thread.id, &ThreadStatus::Paused, &now.to_rfc3339())?;

            let event = AppEvent::ThreadPaused {
                thread_id: thread.id,
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Paused thread: {}", thread.title))
        }

        Intent::Ambiguous => {
            // Treat ambiguous input as a note
            execute_intent(conn, Intent::AddNote, text)
        }

        Intent::Done => {
            let Some(thread) = thread_repo::find_active(conn)? else {
                anyhow::bail!("No active thread or branch to mark done.");
            };

            let _ = branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;
            if let Some(branch) = branch_repo::find_active_for_thread(conn, &thread.id)? {
                if !text.trim().is_empty() {
                    attach_note_to_target(conn, "branch", &branch.id, text)?;
                }
                return mark_branch_done(conn, &thread.id, &branch.id);
            }

            if !text.trim().is_empty() {
                attach_note_to_target(conn, "thread", &thread.id, text)?;
            }

            thread_repo::update_status(conn, &thread.id, &ThreadStatus::Done, &now.to_rfc3339())?;

            let event = AppEvent::ThreadMarkedDone {
                thread_id: thread.id,
                created_at: now,
            };
            event_repo::insert(conn, &event, "tui")?;

            Ok(format!("Done: {}", thread.title))
        }
    }
}

fn execute_intent_with_target(
    conn: &Connection,
    intent: Intent,
    text: &str,
    target: Option<&CommandTarget>,
) -> Result<String> {
    match intent {
        Intent::AddNote => {
            let Some(target) = target else {
                anyhow::bail!("No selected item to attach a note to.");
            };
            attach_note_to_command_target(conn, target, text)?;
            Ok("Note attached.".into())
        }
        Intent::Pause => {
            let Some(target) = target else {
                return execute_intent(conn, intent, text);
            };
            pause_command_target(conn, target, text)
        }
        Intent::Done => {
            let Some(target) = target else {
                return execute_intent(conn, intent, text);
            };
            done_command_target(conn, target, text)
        }
        Intent::ReturnToParent
        | Intent::SetCurrentThread
        | Intent::StartBranch
        | Intent::QueryCurrent
        | Intent::Ambiguous => execute_intent(conn, intent, text),
    }
}

/// Attach a note capture to a specific target.
pub fn attach_note_to_target(
    conn: &Connection,
    target_type: &str,
    target_id: &FlowId,
    text: &str,
) -> Result<()> {
    let now = Utc::now();
    let capture_id = FlowId::new();
    let capture = Capture {
        id: capture_id.clone(),
        target_type: target_type.to_string(),
        target_id: target_id.clone(),
        text: text.to_string(),
        source: CaptureSource::Keyboard,
        inferred_intent: Some(Intent::AddNote),
        created_at: now,
    };
    capture_repo::insert(conn, &capture)?;

    let event = AppEvent::NoteAttached {
        capture_id,
        target_type: target_type.to_string(),
        target_id: target_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(())
}

fn attach_note_to_command_target(
    conn: &Connection,
    target: &CommandTarget,
    text: &str,
) -> Result<()> {
    match target {
        CommandTarget::Thread(thread_id) => attach_note_to_target(conn, "thread", thread_id, text),
        CommandTarget::Branch { branch_id, .. } => {
            attach_note_to_target(conn, "branch", branch_id, text)
        }
    }
}

/// Mark a specific branch done by ID.
pub fn mark_branch_done(
    conn: &Connection,
    thread_id: &FlowId,
    branch_id: &FlowId,
) -> Result<String> {
    let now = Utc::now();
    let Some(branch) = branch_repo::find_by_id(conn, branch_id)? else {
        anyhow::bail!("Branch not found.");
    };

    if branch.status == BranchStatus::Done {
        return Ok(format!("Already done: {}", branch.title));
    }

    branch_repo::update_status(conn, branch_id, &BranchStatus::Done, &now.to_rfc3339())?;

    let event = AppEvent::BranchMarkedDone {
        branch_id: branch_id.clone(),
        thread_id: thread_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(format!("Done: {}", branch.title))
}

/// Mark a specific thread done by ID.
pub fn mark_thread_done(conn: &Connection, thread_id: &FlowId) -> Result<String> {
    mark_thread_done_at(conn, thread_id, Utc::now())
}

fn mark_thread_done_at(
    conn: &Connection,
    thread_id: &FlowId,
    now: DateTime<Utc>,
) -> Result<String> {
    let Some(thread) = thread_repo::find_by_id(conn, thread_id)? else {
        anyhow::bail!("Thread not found.");
    };

    if thread.status == ThreadStatus::Done {
        return Ok(format!("Already done: {}", thread.title));
    }

    mark_child_branches_done(conn, thread_id, now)?;
    thread_repo::update_status(conn, thread_id, &ThreadStatus::Done, &now.to_rfc3339())?;

    let event = AppEvent::ThreadMarkedDone {
        thread_id: thread_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(format!("Done: {}", thread.title))
}

fn mark_child_branches_done(
    conn: &Connection,
    thread_id: &FlowId,
    now: DateTime<Utc>,
) -> Result<()> {
    let branches = branch_repo::find_by_thread(conn, thread_id)?;
    for branch in branches.into_iter().filter(|branch| {
        branch.status != BranchStatus::Archived && branch.status != BranchStatus::Done
    }) {
        branch_repo::update_status(conn, &branch.id, &BranchStatus::Done, &now.to_rfc3339())?;
        let event = AppEvent::BranchMarkedDone {
            branch_id: branch.id,
            thread_id: thread_id.clone(),
            created_at: now,
        };
        event_repo::insert(conn, &event, "tui")?;
    }

    Ok(())
}

/// Archive a specific branch by ID.
pub fn archive_branch(conn: &Connection, thread_id: &FlowId, branch_id: &FlowId) -> Result<String> {
    let now = Utc::now();
    let Some(branch) = branch_repo::find_by_id(conn, branch_id)? else {
        anyhow::bail!("Branch not found.");
    };

    if branch.status == BranchStatus::Archived {
        return Ok(format!("Already archived: {}", branch.title));
    }

    branch_repo::update_status(conn, branch_id, &BranchStatus::Archived, &now.to_rfc3339())?;

    let event = AppEvent::BranchArchived {
        branch_id: branch_id.clone(),
        thread_id: thread_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(format!("Archived: {}", branch.title))
}

/// Archive a specific thread by ID.
pub fn archive_thread(conn: &Connection, thread_id: &FlowId) -> Result<String> {
    let now = Utc::now();
    let Some(thread) = thread_repo::find_by_id(conn, thread_id)? else {
        anyhow::bail!("Thread not found.");
    };

    if thread.status == ThreadStatus::Archived {
        return Ok(format!("Already archived: {}", thread.title));
    }

    thread_repo::update_status(conn, thread_id, &ThreadStatus::Archived, &now.to_rfc3339())?;

    let event = AppEvent::ThreadArchived {
        thread_id: thread_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(format!("Archived: {}", thread.title))
}

fn pause_thread(conn: &Connection, thread_id: &FlowId, note_text: &str) -> Result<String> {
    let now = Utc::now();
    let Some(thread) = thread_repo::find_by_id(conn, thread_id)? else {
        anyhow::bail!("Thread not found.");
    };

    if !note_text.trim().is_empty() {
        attach_note_to_target(conn, "thread", thread_id, note_text)?;
    }

    if thread.status == ThreadStatus::Paused {
        return Ok(format!("Already paused: {}", thread.title));
    }

    thread_repo::update_status(conn, thread_id, &ThreadStatus::Paused, &now.to_rfc3339())?;

    let event = AppEvent::ThreadPaused {
        thread_id: thread_id.clone(),
        created_at: now,
    };
    event_repo::insert(conn, &event, "tui")?;

    Ok(format!("Paused thread: {}", thread.title))
}

pub fn perform_command_on_target(
    conn: &Connection,
    raw: &str,
    target: Option<&CommandTarget>,
) -> InputResult {
    let trimmed = raw.trim();
    if trimmed == "/resume" || trimmed.starts_with("/resume ") {
        let Some(target) = target else {
            return InputResult::Error("No selected item to resume.".into());
        };
        let note_text = trimmed
            .strip_prefix("/resume")
            .unwrap_or("")
            .trim()
            .to_string();
        let result = match target {
            CommandTarget::Thread(thread_id) => resume_thread(conn, thread_id),
            CommandTarget::Branch {
                thread_id,
                branch_id,
            } => resume_branch(conn, thread_id, branch_id),
        };
        if matches!(result, InputResult::Reply(_)) && !note_text.is_empty() {
            if let Err(err) = attach_note_to_command_target(conn, target, &note_text) {
                return InputResult::Error(err.to_string());
            }
        }
        return result;
    }

    if trimmed == "/park" || trimmed.starts_with("/park ") {
        let Some(target) = target else {
            return InputResult::Error("No selected item to park.".into());
        };
        let note_text = trimmed
            .strip_prefix("/park")
            .unwrap_or("")
            .trim()
            .to_string();
        let result = match target {
            CommandTarget::Thread(_) => InputResult::Error("Select a branch to park.".into()),
            CommandTarget::Branch {
                thread_id,
                branch_id,
            } => park_branch(conn, thread_id, branch_id),
        };
        if matches!(result, InputResult::Reply(_)) && !note_text.is_empty() {
            if let Err(err) = attach_note_to_command_target(conn, target, &note_text) {
                return InputResult::Error(err.to_string());
            }
        }
        return result;
    }

    if trimmed == "/archive" || trimmed.starts_with("/archive ") {
        let Some(target) = target else {
            return InputResult::Error("No selected item to archive.".into());
        };
        let note_text = trimmed
            .strip_prefix("/archive")
            .unwrap_or("")
            .trim()
            .to_string();
        let result = match target {
            CommandTarget::Thread(thread_id) => archive_thread(conn, thread_id),
            CommandTarget::Branch {
                thread_id,
                branch_id,
            } => archive_branch(conn, thread_id, branch_id),
        };
        let reply = match result {
            Ok(reply) => reply,
            Err(err) => return InputResult::Error(err.to_string()),
        };
        if !note_text.is_empty() {
            if let Err(err) = attach_note_to_command_target(conn, target, &note_text) {
                return InputResult::Error(err.to_string());
            }
        }
        return InputResult::Reply(reply);
    }

    process_input_with_target(conn, raw, target)
}

fn pause_command_target(conn: &Connection, target: &CommandTarget, text: &str) -> Result<String> {
    match target {
        CommandTarget::Thread(thread_id) => pause_thread(conn, thread_id, text),
        CommandTarget::Branch { thread_id, .. } => pause_thread(conn, thread_id, text),
    }
}

fn done_command_target(conn: &Connection, target: &CommandTarget, text: &str) -> Result<String> {
    match target {
        CommandTarget::Thread(thread_id) => {
            if !text.trim().is_empty() {
                attach_note_to_target(conn, "thread", thread_id, text)?;
            }
            mark_thread_done(conn, thread_id)
        }
        CommandTarget::Branch {
            thread_id,
            branch_id,
        } => {
            if !text.trim().is_empty() {
                attach_note_to_target(conn, "branch", branch_id, text)?;
            }
            mark_branch_done(conn, thread_id, branch_id)
        }
    }
}

/// Resume a specific branch by ID — parks other active branches on the same thread first.
/// Also ensures the parent thread is active.
pub fn resume_branch(conn: &Connection, thread_id: &FlowId, branch_id: &FlowId) -> InputResult {
    let now = Utc::now();

    let parent_is_active = thread_repo::find_by_id(conn, thread_id)
        .ok()
        .flatten()
        .is_some_and(|thread| thread.status == ThreadStatus::Active);

    // Check if the branch is already effectively active
    if let Ok(Some(branch)) = branch_repo::find_by_id(conn, branch_id) {
        if parent_is_active && branch.status == BranchStatus::Active {
            return InputResult::Reply("Branch is already active.".into());
        }
    }

    // Ensure the parent thread is active
    if let Ok(Some(thread)) = thread_repo::find_by_id(conn, thread_id) {
        if thread.status != ThreadStatus::Active {
            // Pause current active thread and activate this one
            if let Ok(Some(current)) = thread_repo::find_active(conn) {
                let _ = thread_repo::update_status(
                    conn,
                    &current.id,
                    &ThreadStatus::Paused,
                    &now.to_rfc3339(),
                );
            }
            let _ = thread_repo::update_status(
                conn,
                thread_id,
                &ThreadStatus::Active,
                &now.to_rfc3339(),
            );
        }
    }

    // Park other active branches on this thread
    if let Ok(branches) = branch_repo::find_by_thread(conn, thread_id) {
        for branch in &branches {
            if branch.status == BranchStatus::Active && branch.id.as_str() != branch_id.as_str() {
                let _ = branch_repo::update_status(
                    conn,
                    &branch.id,
                    &BranchStatus::Parked,
                    &now.to_rfc3339(),
                );
            }
        }
    }

    // Activate the target branch
    if let Err(e) =
        branch_repo::update_status(conn, branch_id, &BranchStatus::Active, &now.to_rfc3339())
    {
        return InputResult::Error(format!("Failed to resume branch: {e}"));
    }

    // Find branch title for the reply message
    let title = branch_repo::find_by_id(conn, branch_id)
        .ok()
        .flatten()
        .map(|b| b.title)
        .unwrap_or_else(|| "unknown".into());

    let event = AppEvent::BranchStarted {
        branch_id: branch_id.clone(),
        thread_id: thread_id.clone(),
        title: title.clone(),
        created_at: now,
    };
    let _ = event_repo::insert(conn, &event, "tui");

    InputResult::Reply(format!("Resumed branch: {title}"))
}

/// Park a specific branch by ID while leaving its parent thread selected as the main focus.
pub fn park_branch(conn: &Connection, thread_id: &FlowId, branch_id: &FlowId) -> InputResult {
    let now = Utc::now();

    let Ok(Some(branch)) = branch_repo::find_by_id(conn, branch_id) else {
        return InputResult::Error("Branch not found.".into());
    };

    if branch.status == BranchStatus::Parked {
        return InputResult::Reply(format!("Branch already parked: {}", branch.title));
    }

    if let Err(e) =
        branch_repo::update_status(conn, branch_id, &BranchStatus::Parked, &now.to_rfc3339())
    {
        return InputResult::Error(format!("Failed to park branch: {e}"));
    }

    let event = AppEvent::BranchParked {
        branch_id: branch_id.clone(),
        thread_id: thread_id.clone(),
        created_at: now,
    };
    let _ = event_repo::insert(conn, &event, "tui");

    InputResult::Reply(format!("Parked branch: {}", branch.title))
}

/// Resume a specific thread by ID — pauses the current active thread first.
pub fn resume_thread(conn: &Connection, thread_id: &FlowId) -> InputResult {
    let now = Utc::now();

    // Pause current active thread if any
    if let Ok(Some(current)) = thread_repo::find_active(conn) {
        if current.id.as_str() == thread_id.as_str() {
            return InputResult::Reply("Thread is already active.".into());
        }
        let _ =
            thread_repo::update_status(conn, &current.id, &ThreadStatus::Paused, &now.to_rfc3339());
    }

    // Activate the target thread
    if let Err(e) =
        thread_repo::update_status(conn, thread_id, &ThreadStatus::Active, &now.to_rfc3339())
    {
        return InputResult::Error(format!("Failed to resume thread: {e}"));
    }

    let _ = branch_repo::normalize_active_for_thread(conn, thread_id, &now.to_rfc3339());

    // Find thread title for the reply message
    let title = thread_repo::find_by_id(conn, thread_id)
        .ok()
        .flatten()
        .map(|t| t.title)
        .unwrap_or_else(|| "unknown".into());

    let event = AppEvent::ThreadSetCurrent {
        thread_id: thread_id.clone(),
        title: title.clone(),
        raw_text: format!("/resume {title}"),
        created_at: now,
    };
    let _ = event_repo::insert(conn, &event, "tui");

    InputResult::Reply(format!("Resumed thread: {title}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use liminal_flow_store::db::open_store_in_memory;
    use liminal_flow_store::repo::{branch_repo, capture_repo, thread_repo};

    fn make_thread(id: &str, title: &str, status: ThreadStatus) -> Thread {
        let now = Utc::now();
        Thread {
            id: FlowId::from(id),
            title: title.into(),
            raw_origin_text: title.into(),
            status,
            short_summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_branch(id: &str, thread_id: &str, title: &str, status: BranchStatus) -> Branch {
        let now = Utc::now();
        Branch {
            id: FlowId::from(id),
            thread_id: FlowId::from(thread_id),
            title: title.into(),
            status,
            short_summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn park_branch_marks_selected_branch_parked() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "answering support", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let result = park_branch(&conn, &thread.id, &branch.id);
        assert!(matches!(result, InputResult::Reply(_)));

        let parked = branch_repo::find_by_id(&conn, &branch.id).unwrap().unwrap();
        assert_eq!(parked.status, BranchStatus::Parked);
    }

    #[test]
    fn resume_branch_reactivates_paused_parent_thread() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Paused);
        let branch = make_branch("b1", "t1", "answering support", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let result = resume_branch(&conn, &thread.id, &branch.id);
        assert!(matches!(result, InputResult::Reply(_)));

        let resumed_thread = thread_repo::find_by_id(&conn, &thread.id).unwrap().unwrap();
        let resumed_branch = branch_repo::find_by_id(&conn, &branch.id).unwrap().unwrap();
        assert_eq!(resumed_thread.status, ThreadStatus::Active);
        assert_eq!(resumed_branch.status, BranchStatus::Active);
    }

    #[test]
    fn add_note_targets_normalized_active_branch() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();
        let thread = Thread {
            id: FlowId::from("t1"),
            title: "improving AIDX".into(),
            raw_origin_text: "improving AIDX".into(),
            status: ThreadStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        };
        let older = Branch {
            id: FlowId::from("b1"),
            thread_id: FlowId::from("t1"),
            title: "older".into(),
            status: BranchStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        };
        let newer = Branch {
            id: FlowId::from("b2"),
            thread_id: FlowId::from("t1"),
            title: "newer".into(),
            status: BranchStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now + chrono::TimeDelta::seconds(5),
        };

        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &older).unwrap();
        branch_repo::upsert(&conn, &newer).unwrap();

        let result = process_input(&conn, "note for the latest branch");
        assert!(matches!(result, InputResult::Reply(_)));

        let older = branch_repo::find_by_id(&conn, &FlowId::from("b1"))
            .unwrap()
            .unwrap();
        let newer = branch_repo::find_by_id(&conn, &FlowId::from("b2"))
            .unwrap()
            .unwrap();
        assert_eq!(older.status, BranchStatus::Parked);
        assert_eq!(newer.status, BranchStatus::Active);

        let newer_captures =
            capture_repo::find_by_target(&conn, "branch", &FlowId::from("b2"), 5).unwrap();
        assert_eq!(newer_captures.len(), 1);
        assert_eq!(newer_captures[0].text, "note for the latest branch");
    }

    #[test]
    fn mark_branch_done_updates_branch_status() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "windows support", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let reply = mark_branch_done(&conn, &thread.id, &branch.id).unwrap();
        assert!(reply.contains("Done"));

        let done_branch = branch_repo::find_by_id(&conn, &branch.id).unwrap().unwrap();
        assert_eq!(done_branch.status, BranchStatus::Done);
    }

    #[test]
    fn mark_thread_done_cascades_to_non_archived_branches() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let parked = make_branch("b1", "t1", "parked branch", BranchStatus::Parked);
        let active = make_branch("b2", "t1", "active branch", BranchStatus::Parked);
        let archived = make_branch("b3", "t1", "archived branch", BranchStatus::Archived);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &parked).unwrap();
        branch_repo::upsert(&conn, &active).unwrap();
        branch_repo::upsert(&conn, &archived).unwrap();

        let reply = mark_thread_done(&conn, &thread.id).unwrap();
        assert!(reply.contains("Done"));

        let thread = thread_repo::find_by_id(&conn, &thread.id).unwrap().unwrap();
        let parked = branch_repo::find_by_id(&conn, &parked.id).unwrap().unwrap();
        let active = branch_repo::find_by_id(&conn, &active.id).unwrap().unwrap();
        let archived = branch_repo::find_by_id(&conn, &archived.id)
            .unwrap()
            .unwrap();
        assert_eq!(thread.status, ThreadStatus::Done);
        assert_eq!(parked.status, BranchStatus::Done);
        assert_eq!(active.status, BranchStatus::Done);
        assert_eq!(archived.status, BranchStatus::Archived);
    }

    #[test]
    fn done_intent_targets_active_branch_before_thread() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "windows support", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let result = process_input(&conn, "/done");
        assert!(matches!(result, InputResult::Reply(_)));

        let thread = thread_repo::find_by_id(&conn, &thread.id).unwrap().unwrap();
        let branch = branch_repo::find_by_id(&conn, &FlowId::from("b1"))
            .unwrap()
            .unwrap();
        assert_eq!(thread.status, ThreadStatus::Active);
        assert_eq!(branch.status, BranchStatus::Done);
    }

    #[test]
    fn unknown_slash_command_errors_instead_of_becoming_note() {
        let conn = open_store_in_memory().unwrap();
        let result = process_input(&conn, "/par definitely not a note");
        assert!(matches!(result, InputResult::Error(_)));
    }

    #[test]
    fn done_command_with_note_attaches_note_to_active_branch() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "windows support", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let result = process_input(&conn, "/done shipped first pass");
        assert!(matches!(result, InputResult::Reply(_)));

        let notes = capture_repo::find_by_target(&conn, "branch", &branch.id, 5).unwrap();
        assert!(notes.iter().any(|note| note.text == "shipped first pass"));
    }

    #[test]
    fn slash_note_targets_selected_branch_without_changing_active_context() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let active_branch = make_branch("b1", "t1", "active branch", BranchStatus::Active);
        let parked_branch = make_branch("b2", "t1", "parked branch", BranchStatus::Parked);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &active_branch).unwrap();
        branch_repo::upsert(&conn, &parked_branch).unwrap();

        let selected = CommandTarget::Branch {
            thread_id: thread.id.clone(),
            branch_id: parked_branch.id.clone(),
        };

        let result = process_input_with_target(&conn, "/note review this later", Some(&selected));
        assert!(matches!(result, InputResult::Reply(_)));

        let active_branch = branch_repo::find_by_id(&conn, &active_branch.id)
            .unwrap()
            .unwrap();
        let parked_branch = branch_repo::find_by_id(&conn, &parked_branch.id)
            .unwrap()
            .unwrap();
        let notes = capture_repo::find_by_target(&conn, "branch", &parked_branch.id, 5).unwrap();

        assert_eq!(active_branch.status, BranchStatus::Active);
        assert_eq!(parked_branch.status, BranchStatus::Parked);
        assert!(notes.iter().any(|note| note.text == "review this later"));
    }

    #[test]
    fn slash_done_targets_selected_branch_before_active_branch() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let active_branch = make_branch("b1", "t1", "active branch", BranchStatus::Active);
        let selected_branch = make_branch("b2", "t1", "selected branch", BranchStatus::Parked);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &active_branch).unwrap();
        branch_repo::upsert(&conn, &selected_branch).unwrap();

        let selected = CommandTarget::Branch {
            thread_id: thread.id.clone(),
            branch_id: selected_branch.id.clone(),
        };

        let result = perform_command_on_target(&conn, "/done wrapped up", Some(&selected));
        assert!(matches!(result, InputResult::Reply(_)));

        let active_branch = branch_repo::find_by_id(&conn, &active_branch.id)
            .unwrap()
            .unwrap();
        let selected_branch = branch_repo::find_by_id(&conn, &selected_branch.id)
            .unwrap()
            .unwrap();
        let notes = capture_repo::find_by_target(&conn, "branch", &selected_branch.id, 5).unwrap();

        assert_eq!(active_branch.status, BranchStatus::Active);
        assert_eq!(selected_branch.status, BranchStatus::Done);
        assert!(notes.iter().any(|note| note.text == "wrapped up"));
    }

    #[test]
    fn slash_pause_targets_selected_branchs_parent_thread() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "selected branch", BranchStatus::Active);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let selected = CommandTarget::Branch {
            thread_id: thread.id.clone(),
            branch_id: branch.id.clone(),
        };

        let result = process_input_with_target(&conn, "/pause waiting on review", Some(&selected));
        assert!(matches!(result, InputResult::Reply(_)));

        let thread = thread_repo::find_by_id(&conn, &thread.id).unwrap().unwrap();
        let notes = capture_repo::find_by_target(&conn, "thread", &thread.id, 5).unwrap();
        assert_eq!(thread.status, ThreadStatus::Paused);
        assert!(notes.iter().any(|note| note.text == "waiting on review"));
    }

    #[test]
    fn archive_thread_updates_thread_status() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Done);
        thread_repo::upsert(&conn, &thread).unwrap();

        let reply = archive_thread(&conn, &thread.id).unwrap();
        assert!(reply.contains("Archived"));

        let archived_thread = thread_repo::find_by_id(&conn, &thread.id).unwrap().unwrap();
        assert_eq!(archived_thread.status, ThreadStatus::Archived);
    }

    #[test]
    fn archive_branch_updates_branch_status() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        let branch = make_branch("b1", "t1", "windows support", BranchStatus::Done);
        thread_repo::upsert(&conn, &thread).unwrap();
        branch_repo::upsert(&conn, &branch).unwrap();

        let reply = archive_branch(&conn, &thread.id, &branch.id).unwrap();
        assert!(reply.contains("Archived"));

        let archived_branch = branch_repo::find_by_id(&conn, &branch.id).unwrap().unwrap();
        assert_eq!(archived_branch.status, BranchStatus::Archived);
    }
}
