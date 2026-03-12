// Database migration runner for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

pub mod v001_initial;
pub mod v002_archive_statuses;

use rusqlite::Connection;

use crate::error::StoreError;

/// Run all pending migrations against the given connection.
pub fn run_migrations(conn: &Connection) -> Result<(), StoreError> {
    let current_version = get_current_version(conn)?;

    type Migration = (i64, &'static str, fn(&Connection) -> Result<(), StoreError>);
    let migrations: Vec<Migration> = vec![
        (1, "initial schema", v001_initial::migrate),
        (2, "archive statuses", v002_archive_statuses::migrate),
    ];

    for (version, name, migrate_fn) in migrations {
        if version > current_version {
            tracing::info!(version, name, "applying migration");
            migrate_fn(conn)?;
        }
    }

    Ok(())
}

/// Get the current schema version, or 0 if the schema_version table doesn't exist.
fn get_current_version(conn: &Connection) -> Result<i64, StoreError> {
    // Check if schema_version table exists
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'schema_version'",
        [],
        |row| row.get(0),
    )?;

    if !table_exists {
        return Ok(0);
    }

    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_run_on_fresh_db() {
        let conn = Connection::open_in_memory().expect("should open in-memory db");
        run_migrations(&conn).expect("migrations should succeed");

        let version = get_current_version(&conn).expect("should get version");
        assert_eq!(version, 2);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().expect("should open in-memory db");
        run_migrations(&conn).expect("first run should succeed");
        run_migrations(&conn).expect("second run should succeed");

        let version = get_current_version(&conn).expect("should get version");
        assert_eq!(version, 2);
    }
}
