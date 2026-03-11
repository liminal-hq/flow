// Migration v001 — initial Liminal Flow schema
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use rusqlite::Connection;

use crate::error::StoreError;

const MIGRATION_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS threads (
    id              TEXT PRIMARY KEY NOT NULL,
    title           TEXT NOT NULL,
    raw_origin_text TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'paused', 'done', 'dropped')),
    short_summary   TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS branches (
    id              TEXT PRIMARY KEY NOT NULL,
    thread_id       TEXT NOT NULL REFERENCES threads(id),
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'parked', 'done', 'dropped')),
    short_summary   TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_branches_thread_id ON branches(thread_id);

CREATE TABLE IF NOT EXISTS captures (
    id              TEXT PRIMARY KEY NOT NULL,
    target_type     TEXT NOT NULL CHECK (target_type IN ('thread', 'branch')),
    target_id       TEXT NOT NULL,
    text            TEXT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'keyboard'
                    CHECK (source IN ('keyboard', 'cli', 'voice', 'import', 'system')),
    inferred_intent TEXT CHECK (inferred_intent IS NULL OR inferred_intent IN (
        'set_current_thread', 'start_branch', 'return_to_parent',
        'add_note', 'query_current', 'pause', 'done', 'ambiguous'
    )),
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_captures_target ON captures(target_type, target_id);

CREATE TABLE IF NOT EXISTS scopes (
    id              TEXT PRIMARY KEY NOT NULL,
    target_type     TEXT NOT NULL CHECK (target_type IN ('thread', 'branch')),
    target_id       TEXT NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('repo', 'cwd', 'git_branch', 'workspace', 'host')),
    value           TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 1.0,
    observed_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_scopes_target ON scopes(target_type, target_id);

CREATE TABLE IF NOT EXISTS hints (
    id              TEXT PRIMARY KEY NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('process', 'command', 'tty', 'activity')),
    value           TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 0.5,
    observed_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
    id              TEXT PRIMARY KEY NOT NULL,
    event_type      TEXT NOT NULL,
    payload_json    TEXT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'cli'
                    CHECK (source IN ('tui', 'cli', 'system', 'infer')),
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_created ON events(created_at);

CREATE TABLE IF NOT EXISTS schema_version (
    version         INTEGER PRIMARY KEY NOT NULL,
    applied_at      TEXT NOT NULL
);
"#;

pub fn migrate(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(MIGRATION_SQL)
        .map_err(|e| StoreError::Migration(format!("v001: {e}")))?;

    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, datetime('now'))",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_all_tables() {
        let conn = Connection::open_in_memory().expect("should open in-memory db");
        migrate(&conn).expect("migration should succeed");

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert!(tables.contains(&"threads".to_string()));
        assert!(tables.contains(&"branches".to_string()));
        assert!(tables.contains(&"captures".to_string()));
        assert!(tables.contains(&"scopes".to_string()));
        assert!(tables.contains(&"hints".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }
}
