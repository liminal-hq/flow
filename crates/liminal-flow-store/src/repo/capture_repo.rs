// Capture repository — CRUD operations for the captures table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{Capture, CaptureSource, FlowId, Intent};
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert a new capture.
pub fn insert(conn: &Connection, capture: &Capture) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO captures (id, target_type, target_id, text, source, inferred_intent, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            capture.id.as_str(),
            capture.target_type,
            capture.target_id.as_str(),
            capture.text,
            capture.source.as_str(),
            capture.inferred_intent.as_ref().map(|i| i.as_str()),
            capture.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Find recent captures for a target (thread or branch).
pub fn find_by_target(
    conn: &Connection,
    target_type: &str,
    target_id: &FlowId,
    limit: usize,
) -> Result<Vec<Capture>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, target_type, target_id, text, source, inferred_intent, created_at
         FROM captures WHERE target_type = ?1 AND target_id = ?2
         ORDER BY created_at DESC LIMIT ?3",
    )?;

    let captures = stmt
        .query_map(
            params![target_type, target_id.as_str(), limit as i64],
            row_to_capture,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(captures)
}

fn row_to_capture(row: &rusqlite::Row) -> rusqlite::Result<Capture> {
    let id: String = row.get(0)?;
    let target_id: String = row.get(2)?;
    let source_str: String = row.get(4)?;
    let intent_str: Option<String> = row.get(5)?;
    let created_str: String = row.get(6)?;

    Ok(Capture {
        id: FlowId::from(id),
        target_type: row.get(1)?,
        target_id: FlowId::from(target_id),
        text: row.get(3)?,
        source: source_str.parse().unwrap_or(CaptureSource::Keyboard),
        inferred_intent: intent_str.and_then(|s| s.parse::<Intent>().ok()),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
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

    #[test]
    fn insert_and_find_captures() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

        // Need a thread first for referential context
        thread_repo::upsert(
            &conn,
            &Thread {
                id: FlowId::from("t1"),
                title: "test".into(),
                raw_origin_text: "test".into(),
                status: ThreadStatus::Active,
                short_summary: None,
                created_at: now,
                updated_at: now,
            },
        )
        .unwrap();

        let capture = Capture {
            id: FlowId::from("c1"),
            target_type: "thread".into(),
            target_id: FlowId::from("t1"),
            text: "I'm improving AIDX".into(),
            source: CaptureSource::Cli,
            inferred_intent: Some(Intent::SetCurrentThread),
            created_at: now,
        };

        insert(&conn, &capture).unwrap();

        let found = find_by_target(&conn, "thread", &FlowId::from("t1"), 10).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "I'm improving AIDX");
        assert_eq!(found[0].source, CaptureSource::Cli);
    }
}
