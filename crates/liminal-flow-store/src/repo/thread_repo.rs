// Thread repository — CRUD operations for the threads table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{FlowId, Thread, ThreadStatus};
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert or update a thread.
pub fn upsert(conn: &Connection, thread: &Thread) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO threads (id, title, raw_origin_text, status, short_summary, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             status = excluded.status,
             short_summary = excluded.short_summary,
             updated_at = excluded.updated_at",
        params![
            thread.id.as_str(),
            thread.title,
            thread.raw_origin_text,
            thread.status.as_str(),
            thread.short_summary,
            thread.created_at.to_rfc3339(),
            thread.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Find the currently active thread, if any.
pub fn find_active(conn: &Connection) -> Result<Option<Thread>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, raw_origin_text, status, short_summary, created_at, updated_at
         FROM threads WHERE status = 'active' LIMIT 1",
    )?;

    let mut rows = stmt.query_map([], row_to_thread)?;
    match rows.next() {
        Some(Ok(thread)) => Ok(Some(thread)),
        Some(Err(e)) => Err(StoreError::Database(e)),
        None => Ok(None),
    }
}

/// Find a thread by ID.
pub fn find_by_id(conn: &Connection, id: &FlowId) -> Result<Option<Thread>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, raw_origin_text, status, short_summary, created_at, updated_at
         FROM threads WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id.as_str()], row_to_thread)?;
    match rows.next() {
        Some(Ok(thread)) => Ok(Some(thread)),
        Some(Err(e)) => Err(StoreError::Database(e)),
        None => Ok(None),
    }
}

/// List all threads with any of the given statuses.
pub fn list_by_statuses(
    conn: &Connection,
    statuses: &[ThreadStatus],
) -> Result<Vec<Thread>, StoreError> {
    if statuses.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<String> = statuses.iter().map(|s| format!("'{}'", s.as_str())).collect();
    let sql = format!(
        "SELECT id, title, raw_origin_text, status, short_summary, created_at, updated_at
         FROM threads WHERE status IN ({}) ORDER BY updated_at DESC",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let threads = stmt.query_map([], row_to_thread)?.collect::<Result<Vec<_>, _>>()?;
    Ok(threads)
}

/// Update a thread's status.
pub fn update_status(
    conn: &Connection,
    id: &FlowId,
    status: &ThreadStatus,
    updated_at: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE threads SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), updated_at, id.as_str()],
    )?;
    Ok(())
}

fn row_to_thread(row: &rusqlite::Row) -> rusqlite::Result<Thread> {
    let id: String = row.get(0)?;
    let status_str: String = row.get(3)?;
    let created_str: String = row.get(5)?;
    let updated_str: String = row.get(6)?;

    Ok(Thread {
        id: FlowId::from(id),
        title: row.get(1)?,
        raw_origin_text: row.get(2)?,
        status: status_str
            .parse()
            .unwrap_or(ThreadStatus::Active),
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
    use chrono::Utc;

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

    #[test]
    fn insert_and_find_active() {
        let conn = open_store_in_memory().unwrap();
        let thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        upsert(&conn, &thread).unwrap();

        let found = find_active(&conn).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "improving AIDX");
    }

    #[test]
    fn find_active_returns_none_when_empty() {
        let conn = open_store_in_memory().unwrap();
        let found = find_active(&conn).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn upsert_updates_existing() {
        let conn = open_store_in_memory().unwrap();
        let mut thread = make_thread("t1", "improving AIDX", ThreadStatus::Active);
        upsert(&conn, &thread).unwrap();

        thread.status = ThreadStatus::Paused;
        upsert(&conn, &thread).unwrap();

        let found = find_by_id(&conn, &FlowId::from("t1")).unwrap().unwrap();
        assert_eq!(found.status, ThreadStatus::Paused);
    }

    #[test]
    fn list_by_statuses_filters() {
        let conn = open_store_in_memory().unwrap();
        upsert(&conn, &make_thread("t1", "active one", ThreadStatus::Active)).unwrap();
        upsert(&conn, &make_thread("t2", "paused one", ThreadStatus::Paused)).unwrap();
        upsert(&conn, &make_thread("t3", "done one", ThreadStatus::Done)).unwrap();

        let active = list_by_statuses(&conn, &[ThreadStatus::Active]).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "active one");

        let active_and_paused =
            list_by_statuses(&conn, &[ThreadStatus::Active, ThreadStatus::Paused]).unwrap();
        assert_eq!(active_and_paused.len(), 2);
    }

    #[test]
    fn update_status_works() {
        let conn = open_store_in_memory().unwrap();
        upsert(&conn, &make_thread("t1", "thread", ThreadStatus::Active)).unwrap();

        update_status(&conn, &FlowId::from("t1"), &ThreadStatus::Done, &Utc::now().to_rfc3339())
            .unwrap();

        let found = find_by_id(&conn, &FlowId::from("t1")).unwrap().unwrap();
        assert_eq!(found.status, ThreadStatus::Done);
    }
}
