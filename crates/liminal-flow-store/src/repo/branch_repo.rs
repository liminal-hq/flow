// Branch repository — CRUD operations for the branches table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{Branch, BranchStatus, FlowId};
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert or update a branch.
pub fn upsert(conn: &Connection, branch: &Branch) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO branches (id, thread_id, title, status, short_summary, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             status = excluded.status,
             short_summary = excluded.short_summary,
             updated_at = excluded.updated_at",
        params![
            branch.id.as_str(),
            branch.thread_id.as_str(),
            branch.title,
            branch.status.as_str(),
            branch.short_summary,
            branch.created_at.to_rfc3339(),
            branch.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Find all branches for a given thread.
pub fn find_by_thread(conn: &Connection, thread_id: &FlowId) -> Result<Vec<Branch>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, thread_id, title, status, short_summary, created_at, updated_at
         FROM branches WHERE thread_id = ?1 ORDER BY created_at ASC",
    )?;

    let branches = stmt
        .query_map(params![thread_id.as_str()], row_to_branch)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(branches)
}

/// Find the active branch for a given thread, if any.
pub fn find_active_for_thread(
    conn: &Connection,
    thread_id: &FlowId,
) -> Result<Option<Branch>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, thread_id, title, status, short_summary, created_at, updated_at
         FROM branches WHERE thread_id = ?1 AND status = 'active' LIMIT 1",
    )?;

    let mut rows = stmt.query_map(params![thread_id.as_str()], row_to_branch)?;
    match rows.next() {
        Some(Ok(branch)) => Ok(Some(branch)),
        Some(Err(e)) => Err(StoreError::Database(e)),
        None => Ok(None),
    }
}

/// Find a branch by ID.
pub fn find_by_id(conn: &Connection, id: &FlowId) -> Result<Option<Branch>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, thread_id, title, status, short_summary, created_at, updated_at
         FROM branches WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id.as_str()], row_to_branch)?;
    match rows.next() {
        Some(Ok(branch)) => Ok(Some(branch)),
        Some(Err(e)) => Err(StoreError::Database(e)),
        None => Ok(None),
    }
}

/// Update a branch's status.
pub fn update_status(
    conn: &Connection,
    id: &FlowId,
    status: &BranchStatus,
    updated_at: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE branches SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), updated_at, id.as_str()],
    )?;
    Ok(())
}

fn row_to_branch(row: &rusqlite::Row) -> rusqlite::Result<Branch> {
    let id: String = row.get(0)?;
    let thread_id: String = row.get(1)?;
    let status_str: String = row.get(3)?;
    let created_str: String = row.get(5)?;
    let updated_str: String = row.get(6)?;

    Ok(Branch {
        id: FlowId::from(id),
        thread_id: FlowId::from(thread_id),
        title: row.get(2)?,
        status: status_str.parse().unwrap_or(BranchStatus::Active),
        short_summary: row.get(4)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .unwrap_or_default()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
            .unwrap_or_default()
            .with_timezone(&chrono::Utc),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_store_in_memory;
    use crate::repo::thread_repo;
    use chrono::Utc;
    use liminal_flow_core::model::{Thread, ThreadStatus};

    fn make_thread(id: &str) -> Thread {
        let now = Utc::now();
        Thread {
            id: FlowId::from(id),
            title: "test thread".into(),
            raw_origin_text: "test".into(),
            status: ThreadStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_branch(id: &str, thread_id: &str, title: &str) -> Branch {
        let now = Utc::now();
        Branch {
            id: FlowId::from(id),
            thread_id: FlowId::from(thread_id),
            title: title.into(),
            status: BranchStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn insert_and_find_by_thread() {
        let conn = open_store_in_memory().unwrap();
        thread_repo::upsert(&conn, &make_thread("t1")).unwrap();

        upsert(&conn, &make_branch("b1", "t1", "answering support")).unwrap();
        upsert(&conn, &make_branch("b2", "t1", "reading article")).unwrap();

        let branches = find_by_thread(&conn, &FlowId::from("t1")).unwrap();
        assert_eq!(branches.len(), 2);
    }

    #[test]
    fn find_active_for_thread_works() {
        let conn = open_store_in_memory().unwrap();
        thread_repo::upsert(&conn, &make_thread("t1")).unwrap();

        let mut b1 = make_branch("b1", "t1", "parked one");
        b1.status = BranchStatus::Parked;
        upsert(&conn, &b1).unwrap();
        upsert(&conn, &make_branch("b2", "t1", "active one")).unwrap();

        let active = find_active_for_thread(&conn, &FlowId::from("t1")).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().title, "active one");
    }

    #[test]
    fn update_branch_status() {
        let conn = open_store_in_memory().unwrap();
        thread_repo::upsert(&conn, &make_thread("t1")).unwrap();
        upsert(&conn, &make_branch("b1", "t1", "branch")).unwrap();

        update_status(
            &conn,
            &FlowId::from("b1"),
            &BranchStatus::Parked,
            &Utc::now().to_rfc3339(),
        )
        .unwrap();

        let branches = find_by_thread(&conn, &FlowId::from("t1")).unwrap();
        assert_eq!(branches[0].status, BranchStatus::Parked);
    }
}
