// Hint repository — CRUD operations for the hints table
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{FlowId, Hint, HintKind};
use rusqlite::{params, Connection};

use crate::error::StoreError;

/// Insert a new hint.
pub fn insert(conn: &Connection, hint: &Hint) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO hints (id, kind, value, confidence, observed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            hint.id.as_str(),
            hint.kind.as_str(),
            hint.value,
            hint.confidence,
            hint.observed_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Find recent hints.
pub fn find_recent(conn: &Connection, limit: usize) -> Result<Vec<Hint>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, value, confidence, observed_at
         FROM hints ORDER BY observed_at DESC LIMIT ?1",
    )?;

    let hints = stmt
        .query_map(params![limit as i64], |row| {
            let id: String = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let observed_str: String = row.get(4)?;

            Ok(Hint {
                id: FlowId::from(id),
                kind: kind_str.parse().unwrap_or(HintKind::Activity),
                value: row.get(2)?,
                confidence: row.get(3)?,
                observed_at: chrono::DateTime::parse_from_rfc3339(&observed_str)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(hints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_store_in_memory;
    use chrono::Utc;

    #[test]
    fn insert_and_find_hints() {
        let conn = open_store_in_memory().unwrap();

        let hint = Hint {
            id: FlowId::new(),
            kind: HintKind::Process,
            value: "cargo test running".into(),
            confidence: 0.5,
            observed_at: Utc::now(),
        };

        insert(&conn, &hint).unwrap();

        let found = find_recent(&conn, 10).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, HintKind::Process);
    }
}
