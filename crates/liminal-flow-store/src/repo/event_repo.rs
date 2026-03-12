// Event repository — insert and query operations for the events table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::event::AppEvent;
use liminal_flow_core::model::FlowId;
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert a domain event into the events table.
pub fn insert(conn: &Connection, event: &AppEvent, source: &str) -> Result<(), StoreError> {
    let id = FlowId::new();
    let event_type = event.event_type();
    let payload_json =
        serde_json::to_string(event).map_err(|e| StoreError::Migration(e.to_string()))?;
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO events (id, event_type, payload_json, source, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id.as_str(), event_type, payload_json, source, created_at],
    )?;
    Ok(())
}

/// Check whether any events have been created after the given timestamp.
/// Used by the TUI for polling.
pub fn has_events_after(conn: &Connection, after: &str) -> Result<bool, StoreError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events WHERE created_at > ?1",
        params![after],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Get the latest event timestamp, or None if no events exist.
pub fn latest_timestamp(conn: &Connection) -> Result<Option<String>, StoreError> {
    let result: Option<String> =
        conn.query_row("SELECT MAX(created_at) FROM events", [], |row| row.get(0))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_store_in_memory;
    use liminal_flow_core::event::AppEvent;
    use liminal_flow_core::model::{CaptureSource, FlowId};

    #[test]
    fn insert_and_check_events() {
        let conn = open_store_in_memory().unwrap();

        let before = chrono::Utc::now().to_rfc3339();

        let event = AppEvent::ThreadSetCurrent {
            thread_id: FlowId::from("t1"),
            title: "improving AIDX".into(),
            raw_text: "improving AIDX".into(),
            created_at: chrono::Utc::now(),
        };

        insert(&conn, &event, "cli").unwrap();

        assert!(has_events_after(&conn, &before).unwrap());

        let latest = latest_timestamp(&conn).unwrap();
        assert!(latest.is_some());
    }

    #[test]
    fn no_events_after_future_timestamp() {
        let conn = open_store_in_memory().unwrap();

        let event = AppEvent::CaptureReceived {
            capture_id: FlowId::new(),
            text: "test".into(),
            source: CaptureSource::Cli,
            created_at: chrono::Utc::now(),
        };

        insert(&conn, &event, "cli").unwrap();

        // Check with a far-future timestamp
        assert!(!has_events_after(&conn, "9999-12-31T23:59:59Z").unwrap());
    }
}
