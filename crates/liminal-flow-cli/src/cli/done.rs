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

    let branches = branch_repo::find_by_thread(conn, &thread.id)?;
    for branch in branches.into_iter().filter(|branch| {
        branch.status != BranchStatus::Archived && branch.status != BranchStatus::Done
    }) {
        branch_repo::update_status(conn, &branch.id, &BranchStatus::Done, &now.to_rfc3339())?;
        let event = AppEvent::BranchMarkedDone {
            branch_id: branch.id,
            thread_id: thread.id.clone(),
            created_at: now,
        };
        event_repo::insert(conn, &event, "cli")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use liminal_flow_core::model::{Branch, FlowId, Thread};
    use liminal_flow_store::db::open_store_in_memory;

    #[test]
    fn marking_thread_done_also_marks_non_archived_branches_done() {
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

        for (id, title, status) in [
            ("b1", "parked", BranchStatus::Parked),
            ("b2", "active", BranchStatus::Active),
            ("b3", "archived", BranchStatus::Archived),
        ] {
            branch_repo::upsert(
                &conn,
                &Branch {
                    id: FlowId::from(id),
                    thread_id: FlowId::from("t1"),
                    title: title.into(),
                    status,
                    short_summary: None,
                    created_at: now,
                    updated_at: now,
                },
            )
            .unwrap();
        }

        branch_repo::update_status(
            &conn,
            &FlowId::from("b2"),
            &BranchStatus::Parked,
            &now.to_rfc3339(),
        )
        .unwrap();

        handle(&conn).unwrap();

        let thread = thread_repo::find_by_id(&conn, &FlowId::from("t1"))
            .unwrap()
            .unwrap();
        let parked = branch_repo::find_by_id(&conn, &FlowId::from("b1"))
            .unwrap()
            .unwrap();
        let formerly_active = branch_repo::find_by_id(&conn, &FlowId::from("b2"))
            .unwrap()
            .unwrap();
        let archived = branch_repo::find_by_id(&conn, &FlowId::from("b3"))
            .unwrap()
            .unwrap();

        assert_eq!(thread.status, ThreadStatus::Done);
        assert_eq!(parked.status, BranchStatus::Done);
        assert_eq!(formerly_active.status, BranchStatus::Done);
        assert_eq!(archived.status, BranchStatus::Archived);
    }
}
