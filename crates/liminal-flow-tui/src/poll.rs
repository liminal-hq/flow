// Database change detection for TUI refresh
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_store::repo::event_repo;
use rusqlite::Connection;

/// Check if the database has changed since the last watermark.
/// Returns `true` if new events exist (or if this is the first poll).
pub fn has_changes(conn: &Connection, watermark: &Option<String>) -> bool {
    match watermark {
        None => true,
        Some(ts) => event_repo::has_events_after(conn, ts).unwrap_or(false),
    }
}

/// Get the current watermark (latest event timestamp).
pub fn current_watermark(conn: &Connection) -> Option<String> {
    event_repo::latest_timestamp(conn).unwrap_or(None)
}
