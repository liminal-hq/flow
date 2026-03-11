// TUI state management
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{Branch, Thread, ThreadStatus};
use liminal_flow_store::repo::{branch_repo, scope_repo, thread_repo};
use rusqlite::Connection;

/// Interaction mode for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Help,
}

/// A thread together with its branches, for display in the thread list.
#[derive(Debug, Clone)]
pub struct ThreadEntry {
    pub thread: Thread,
    pub branches: Vec<Branch>,
}

/// Scope context displayed in the reply pane.
#[derive(Debug, Clone, Default)]
pub struct ScopeContext {
    pub repo: Option<String>,
    pub git_branch: Option<String>,
    pub cwd: Option<String>,
}

/// The full TUI state, refreshed from the database on each poll tick.
pub struct TuiState {
    pub mode: Mode,
    pub threads: Vec<ThreadEntry>,
    pub selected_index: usize,
    pub last_reply: Option<String>,
    pub error_message: Option<String>,
    pub scope_context: ScopeContext,
    pub poll_watermark: Option<String>,
    pub should_quit: bool,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Insert,
            threads: Vec::new(),
            selected_index: 0,
            last_reply: None,
            error_message: None,
            scope_context: ScopeContext::default(),
            poll_watermark: None,
            should_quit: false,
        }
    }

    /// Refresh state from the database.
    pub fn refresh_from_db(&mut self, conn: &Connection) {
        // Load active and paused threads
        let threads = thread_repo::list_by_statuses(
            conn,
            &[ThreadStatus::Active, ThreadStatus::Paused],
        )
        .unwrap_or_default();

        self.threads = threads
            .into_iter()
            .map(|thread| {
                let branches = branch_repo::find_by_thread(conn, &thread.id)
                    .unwrap_or_default();
                ThreadEntry { thread, branches }
            })
            .collect();

        // Load scope context for the active thread
        self.scope_context = ScopeContext::default();
        if let Some(active) = self.active_thread() {
            let scopes = scope_repo::find_by_target(conn, "thread", &active.thread.id)
                .unwrap_or_default();
            for scope in &scopes {
                match scope.kind {
                    liminal_flow_core::model::ScopeKind::Repo => {
                        self.scope_context.repo = Some(scope.value.clone());
                    }
                    liminal_flow_core::model::ScopeKind::GitBranch => {
                        self.scope_context.git_branch = Some(scope.value.clone());
                    }
                    liminal_flow_core::model::ScopeKind::Cwd => {
                        self.scope_context.cwd = Some(scope.value.clone());
                    }
                    _ => {}
                }
            }
        }

        // Clamp selected index
        if !self.threads.is_empty() && self.selected_index >= self.threads.len() {
            self.selected_index = self.threads.len() - 1;
        }
    }

    /// Return the active thread entry, if any.
    pub fn active_thread(&self) -> Option<&ThreadEntry> {
        self.threads.iter().find(|e| e.thread.status == ThreadStatus::Active)
    }

    pub fn select_next(&mut self) {
        if !self.threads.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.threads.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.threads.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.threads.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}
