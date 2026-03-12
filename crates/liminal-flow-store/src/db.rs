// Database connection setup for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::path::Path;

use rusqlite::Connection;

use crate::error::StoreError;
use crate::migrations;

/// Open or create the SQLite database, run migrations, and return the connection.
///
/// For production use, this opens the database at the platform-appropriate path.
/// For testing, use `open_store_at` with a custom path or `open_store_in_memory`.
pub fn open_store() -> Result<Connection, StoreError> {
    let db_path = crate::paths::database_path()?;

    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            StoreError::PathResolution(format!("could not create data directory: {e}"))
        })?;
    }

    open_store_at(&db_path)
}

/// Open or create the SQLite database at a specific path.
pub fn open_store_at(path: &Path) -> Result<Connection, StoreError> {
    let conn = Connection::open(path)?;
    configure_connection(&conn)?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}

/// Open an in-memory SQLite database (for testing).
pub fn open_store_in_memory() -> Result<Connection, StoreError> {
    let conn = Connection::open_in_memory()?;
    configure_connection(&conn)?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}

/// Apply recommended SQLite pragmas.
fn configure_connection(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_store_opens_and_migrates() {
        let conn = open_store_in_memory().expect("should open in-memory store");

        // Verify WAL mode is set
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("should query journal mode");
        // In-memory databases may report "memory" instead of "wal"
        assert!(
            journal_mode == "wal" || journal_mode == "memory",
            "unexpected journal mode: {journal_mode}"
        );

        // Verify schema version
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .expect("should query schema version");
        assert_eq!(version, 2);
    }
}
