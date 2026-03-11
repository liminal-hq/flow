// Handler for `flo list` — list active and paused threads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use anyhow::Result;
use liminal_flow_core::model::{BranchStatus, Intent, ThreadStatus};
use liminal_flow_store::repo::{branch_repo, capture_repo, thread_repo};
use rusqlite::Connection;

use super::colour;

pub fn handle(conn: &Connection, show_all: bool) -> Result<()> {
    let threads =
        thread_repo::list_by_statuses(conn, &[ThreadStatus::Active, ThreadStatus::Paused])?;

    if threads.is_empty() {
        println!("{}", colour::muted("No active threads."));
        return Ok(());
    }

    for thread in &threads {
        let is_active = thread.status == ThreadStatus::Active;

        // Thread line: "> title" for active, "  title  paused" for paused
        let marker = if is_active {
            colour::green(">")
        } else {
            " ".to_string()
        };
        let title = if is_active {
            colour::bold(&colour::green(&thread.title))
        } else {
            thread.title.clone()
        };
        let status_label = if thread.status == ThreadStatus::Paused {
            format!("  {}", colour::muted("paused"))
        } else {
            String::new()
        };

        println!("{marker} {title}{status_label}");

        if show_all {
            // Show branches
            let branches = branch_repo::find_by_thread(conn, &thread.id)?;
            for branch in &branches {
                let branch_marker = if branch.status == BranchStatus::Active {
                    colour::blue("*")
                } else {
                    " ".to_string()
                };
                let branch_title = if branch.status == BranchStatus::Active {
                    colour::blue(&branch.title)
                } else {
                    colour::muted(&branch.title)
                };
                let branch_status = if branch.status != BranchStatus::Active {
                    format!("  {}", colour::muted(&format!("({})", branch.status)))
                } else {
                    String::new()
                };
                println!("    {branch_marker} {branch_title}{branch_status}");
            }

            // Show recent notes
            let captures = capture_repo::find_by_target(conn, "thread", &thread.id, 3)?;
            let notes: Vec<_> = captures
                .iter()
                .filter(|c| {
                    c.inferred_intent
                        .as_ref()
                        .is_some_and(|i| *i == Intent::AddNote)
                })
                .collect();

            for note in &notes {
                println!("    {} {}", colour::muted("|"), colour::muted(&note.text));
            }
        }
    }

    Ok(())
}
