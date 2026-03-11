// Handler for `flo where` — print current thread and branches
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use liminal_flow_store::repo::{branch_repo, thread_repo};
use rusqlite::Connection;

pub fn handle(conn: &Connection) -> Result<()> {
    let current_thread = thread_repo::find_active(conn)?;

    let Some(thread) = current_thread else {
        println!("(no active thread)");
        return Ok(());
    };

    println!("Current thread: {}", thread.title);

    let branches = branch_repo::find_by_thread(conn, &thread.id)?;
    if !branches.is_empty() {
        let descriptions: Vec<String> = branches
            .iter()
            .map(|b| {
                if b.status == liminal_flow_core::model::BranchStatus::Active {
                    b.title.clone()
                } else {
                    format!("{} ({})", b.title, b.status)
                }
            })
            .collect();
        println!("Branches: {}", descriptions.join(", "));
    }

    Ok(())
}
