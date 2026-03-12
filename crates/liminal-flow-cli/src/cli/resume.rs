// Handler for `flo resume` — revive the most recent resumable item
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

    if let Some(active_thread) = thread_repo::find_active(conn)? {
        let mut resumable_branches: Vec<_> =
            branch_repo::find_visible_by_thread(conn, &active_thread.id)?
                .into_iter()
                .filter(|branch| matches!(branch.status, BranchStatus::Parked | BranchStatus::Done))
                .collect();
        resumable_branches.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

        if let Some(branch) = resumable_branches.into_iter().next() {
            let branches = branch_repo::find_by_thread(conn, &active_thread.id)?;
            for existing in &branches {
                if existing.status == BranchStatus::Active && existing.id != branch.id {
                    branch_repo::update_status(
                        conn,
                        &existing.id,
                        &BranchStatus::Parked,
                        &now.to_rfc3339(),
                    )?;
                }
            }

            branch_repo::update_status(conn, &branch.id, &BranchStatus::Active, &now.to_rfc3339())?;

            let event = AppEvent::BranchStarted {
                branch_id: branch.id,
                thread_id: active_thread.id,
                title: branch.title.clone(),
                created_at: now,
            };
            event_repo::insert(conn, &event, "cli")?;

            println!("Resumed branch: {}", branch.title);
            return Ok(());
        }

        bail!(
            "An active thread is already in focus. No parked or done branch is available to resume on it."
        );
    }

    let mut resumable_threads =
        thread_repo::list_by_statuses(conn, &[ThreadStatus::Paused, ThreadStatus::Done])?;
    resumable_threads.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let Some(thread) = resumable_threads.into_iter().next() else {
        bail!("No paused or done item to resume.");
    };

    if let Some(current) = thread_repo::find_active(conn)? {
        if current.id != thread.id {
            thread_repo::update_status(
                conn,
                &current.id,
                &ThreadStatus::Paused,
                &now.to_rfc3339(),
            )?;
        }
    }

    thread_repo::update_status(conn, &thread.id, &ThreadStatus::Active, &now.to_rfc3339())?;
    let _ = branch_repo::normalize_active_for_thread(conn, &thread.id, &now.to_rfc3339())?;

    let event = AppEvent::ThreadSetCurrent {
        thread_id: thread.id,
        title: thread.title.clone(),
        raw_text: format!("/resume {}", thread.title),
        created_at: now,
    };
    event_repo::insert(conn, &event, "cli")?;

    println!("Resumed thread: {}", thread.title);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use liminal_flow_core::model::{Branch, FlowId, Thread};
    use liminal_flow_store::db::open_store_in_memory;

    #[test]
    fn resume_prefers_branch_on_active_thread() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t1"),
                title: "thread".into(),
                raw_origin_text: "thread".into(),
                status: ThreadStatus::Active,
                short_summary: None,
                created_at: now,
                updated_at: now,
            },
        )
        .unwrap();
        branch_repo::upsert(
            &conn,
            &Branch {
                id: FlowId::from("b1"),
                thread_id: FlowId::from("t1"),
                title: "branch".into(),
                status: BranchStatus::Done,
                short_summary: None,
                created_at: now,
                updated_at: now,
            },
        )
        .unwrap();

        handle(&conn).unwrap();

        let branch = branch_repo::find_by_id(&conn, &FlowId::from("b1"))
            .unwrap()
            .unwrap();
        assert_eq!(branch.status, BranchStatus::Active);
    }

    #[test]
    fn resume_revives_most_recent_thread_without_active_thread() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t1"),
                title: "thread one".into(),
                raw_origin_text: "thread one".into(),
                status: ThreadStatus::Done,
                short_summary: None,
                created_at: now,
                updated_at: now,
            },
        )
        .unwrap();
        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t2"),
                title: "thread two".into(),
                raw_origin_text: "thread two".into(),
                status: ThreadStatus::Paused,
                short_summary: None,
                created_at: now,
                updated_at: now + chrono::TimeDelta::seconds(5),
            },
        )
        .unwrap();

        handle(&conn).unwrap();

        let resumed = thread_repo::find_by_id(&conn, &FlowId::from("t2"))
            .unwrap()
            .unwrap();
        assert_eq!(resumed.status, ThreadStatus::Active);
    }

    #[test]
    fn resume_does_not_switch_to_another_thread_when_one_is_active() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t1"),
                title: "active thread".into(),
                raw_origin_text: "active thread".into(),
                status: ThreadStatus::Active,
                short_summary: None,
                created_at: now,
                updated_at: now,
            },
        )
        .unwrap();
        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t2"),
                title: "paused thread".into(),
                raw_origin_text: "paused thread".into(),
                status: ThreadStatus::Paused,
                short_summary: None,
                created_at: now,
                updated_at: now + chrono::TimeDelta::seconds(5),
            },
        )
        .unwrap();

        let error = handle(&conn).unwrap_err().to_string();
        assert!(error.contains("No parked or done branch"));

        let active = thread_repo::find_active(&conn).unwrap().unwrap();
        assert_eq!(active.id, FlowId::from("t1"));

        let paused = thread_repo::find_by_id(&conn, &FlowId::from("t2"))
            .unwrap()
            .unwrap();
        assert_eq!(paused.status, ThreadStatus::Paused);
    }
}
