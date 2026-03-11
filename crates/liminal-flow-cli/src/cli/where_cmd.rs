// Handler for `flo where` — print current thread and branches
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use liminal_flow_core::model::{BranchStatus, Intent};
use liminal_flow_store::repo::{branch_repo, capture_repo, scope_repo, thread_repo};
use rusqlite::Connection;

use super::colour;

pub fn handle(conn: &Connection) -> Result<()> {
    let current_thread = thread_repo::find_active(conn)?;

    let Some(thread) = current_thread else {
        println!("{}", colour::muted("(no active thread)"));
        return Ok(());
    };

    // Thread header
    println!(
        "{} {}",
        colour::muted("Thread:"),
        colour::bold(&colour::green(&thread.title))
    );

    // Branches
    let branches = branch_repo::find_by_thread(conn, &thread.id)?;
    if !branches.is_empty() {
        for branch in &branches {
            let marker = if branch.status == BranchStatus::Active {
                colour::blue("*")
            } else {
                " ".to_string()
            };
            let title = if branch.status == BranchStatus::Active {
                colour::blue(&branch.title)
            } else {
                colour::muted(&branch.title)
            };
            let status = if branch.status != BranchStatus::Active {
                format!("  {}", colour::muted(&format!("({})", branch.status)))
            } else {
                String::new()
            };
            println!("  {marker} {title}{status}");
        }
    }

    // Scope context
    let scopes = scope_repo::find_by_target(conn, "thread", &thread.id)?;
    let mut scope_parts: Vec<String> = Vec::new();
    for scope in &scopes {
        match scope.kind {
            liminal_flow_core::model::ScopeKind::Repo => {
                scope_parts.push(format!(
                    "{} {}",
                    colour::muted("Repo:"),
                    scope.value
                ));
            }
            liminal_flow_core::model::ScopeKind::GitBranch => {
                scope_parts.push(format!(
                    "{} {}",
                    colour::muted("Git:"),
                    scope.value
                ));
            }
            liminal_flow_core::model::ScopeKind::Cwd => {
                scope_parts.push(format!(
                    "{} {}",
                    colour::muted("Dir:"),
                    scope.value
                ));
            }
            _ => {}
        }
    }
    if !scope_parts.is_empty() {
        for part in &scope_parts {
            println!("  {part}");
        }
    }

    // Recent notes
    let captures = capture_repo::find_by_target(conn, "thread", &thread.id, 5)?;
    let notes: Vec<_> = captures
        .iter()
        .filter(|c| {
            c.inferred_intent
                .as_ref()
                .is_some_and(|i| *i == Intent::AddNote)
        })
        .collect();

    if !notes.is_empty() {
        println!();
        println!("{}", colour::muted("Notes:"));
        for note in &notes {
            println!("  {} {}", colour::muted("|"), note.text);
        }
    }

    Ok(())
}
