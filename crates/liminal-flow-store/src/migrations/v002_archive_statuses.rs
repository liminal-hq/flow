// Migration v002 — add archived lifecycle support to threads and branches
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use rusqlite::Connection;

use crate::error::StoreError;

pub fn migrate(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;
        BEGIN TRANSACTION;

        ALTER TABLE threads RENAME TO threads_old;
        CREATE TABLE threads (
            id              TEXT PRIMARY KEY NOT NULL,
            title           TEXT NOT NULL,
            raw_origin_text TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'active'
                            CHECK (status IN ('active', 'paused', 'done', 'archived', 'dropped')),
            short_summary   TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );
        INSERT INTO threads (id, title, raw_origin_text, status, short_summary, created_at, updated_at)
        SELECT id, title, raw_origin_text, status, short_summary, created_at, updated_at
        FROM threads_old;
        DROP TABLE threads_old;

        ALTER TABLE branches RENAME TO branches_old;
        CREATE TABLE branches (
            id              TEXT PRIMARY KEY NOT NULL,
            thread_id       TEXT NOT NULL REFERENCES threads(id),
            title           TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'active'
                            CHECK (status IN ('active', 'parked', 'done', 'archived', 'dropped')),
            short_summary   TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        );
        INSERT INTO branches (id, thread_id, title, status, short_summary, created_at, updated_at)
        SELECT id, thread_id, title, status, short_summary, created_at, updated_at
        FROM branches_old;
        DROP TABLE branches_old;
        CREATE INDEX IF NOT EXISTS idx_branches_thread_id ON branches(thread_id);

        INSERT INTO schema_version (version, applied_at) VALUES (2, datetime('now'));

        COMMIT;
        PRAGMA foreign_keys = ON;
        "#,
    )
    .map_err(|e| StoreError::Migration(format!("v002: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_accepts_archived_statuses() {
        let conn = Connection::open_in_memory().expect("should open in-memory db");
        crate::migrations::v001_initial::migrate(&conn).expect("v001 should succeed");
        migrate(&conn).expect("v002 should succeed");

        conn.execute(
            "INSERT INTO threads (id, title, raw_origin_text, status, created_at, updated_at)
             VALUES ('t1', 'archived thread', 'archived thread', 'archived', datetime('now'), datetime('now'))",
            [],
        )
        .expect("thread should accept archived status");

        conn.execute(
            "INSERT INTO branches (id, thread_id, title, status, created_at, updated_at)
             VALUES ('b1', 't1', 'archived branch', 'archived', datetime('now'), datetime('now'))",
            [],
        )
        .expect("branch should accept archived status");
    }
}
