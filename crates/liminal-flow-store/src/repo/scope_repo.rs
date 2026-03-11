// Scope repository — CRUD operations for the scopes table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{FlowId, Scope, ScopeKind};
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert a new scope.
pub fn insert(conn: &Connection, scope: &Scope) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO scopes (id, target_type, target_id, kind, value, confidence, observed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            scope.id.as_str(),
            scope.target_type,
            scope.target_id.as_str(),
            scope.kind.as_str(),
            scope.value,
            scope.confidence,
            scope.observed_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Find all scopes for a target.
pub fn find_by_target(
    conn: &Connection,
    target_type: &str,
    target_id: &FlowId,
) -> Result<Vec<Scope>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, target_type, target_id, kind, value, confidence, observed_at
         FROM scopes WHERE target_type = ?1 AND target_id = ?2
         ORDER BY observed_at DESC",
    )?;

    let scopes = stmt
        .query_map(params![target_type, target_id.as_str()], |row| {
            let id: String = row.get(0)?;
            let target_id: String = row.get(2)?;
            let kind_str: String = row.get(3)?;
            let observed_str: String = row.get(6)?;

            Ok(Scope {
                id: FlowId::from(id),
                target_type: row.get(1)?,
                target_id: FlowId::from(target_id),
                kind: kind_str.parse().unwrap_or(ScopeKind::Cwd),
                value: row.get(4)?,
                confidence: row.get(5)?,
                observed_at: chrono::DateTime::parse_from_rfc3339(&observed_str)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(scopes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_store_in_memory;
    use crate::repo::thread_repo;
    use chrono::Utc;
    use liminal_flow_core::model::{Thread, ThreadStatus};

    #[test]
    fn insert_and_find_scopes() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

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

        let scope = Scope {
            id: FlowId::new(),
            target_type: "thread".into(),
            target_id: FlowId::from("t1"),
            kind: ScopeKind::Repo,
            value: "/home/user/project".into(),
            confidence: 0.8,
            observed_at: now,
        };

        insert(&conn, &scope).unwrap();

        let found = find_by_target(&conn, "thread", &FlowId::from("t1")).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ScopeKind::Repo);
        assert_eq!(found[0].value, "/home/user/project");
    }
}
