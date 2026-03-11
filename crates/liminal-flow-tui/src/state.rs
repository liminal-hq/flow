// TUI state management
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use liminal_flow_core::model::{Branch, Capture, FlowId, Thread, ThreadStatus};
use liminal_flow_store::repo::{branch_repo, capture_repo, scope_repo, thread_repo};
use rusqlite::Connection;

/// Interaction mode for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Help,
    About,
}

/// Identifies a selectable item in the thread list — either a thread or a branch within it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedItem {
    Thread(usize),
    Branch(usize, usize), // (thread_index, branch_index)
}

/// Slash commands available in the command palette.
pub const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/now <text>", "Set or replace the current thread"),
    ("/branch <text>", "Start a branch beneath current thread"),
    ("/back", "Return to the parent thread"),
    ("/note <text>", "Attach a note (or just type plain text)"),
    ("/where", "Show current thread and branches"),
    ("/pause", "Pause the current thread"),
    ("/done", "Mark the current thread done"),
];

/// Keyboard shortcut hints shown when ? is typed on an empty line.
pub const SHORTCUT_HINTS: &[(&str, &str)] = &[
    ("/ for commands", "Esc to Normal mode"),
    ("Enter to submit/expand", "i to Insert mode"),
    ("? for shortcuts", "q to quit (Normal)"),
    ("Up/Down navigate items", "r resume selected (Normal)"),
];

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
    /// The currently selected item in the thread list (thread or branch).
    pub selected: SelectedItem,
    pub last_reply: Option<String>,
    pub error_message: Option<String>,
    pub scope_context: ScopeContext,
    pub poll_watermark: Option<String>,
    pub should_quit: bool,
    /// Whether the command palette popup is visible (triggered by `/` on empty line).
    pub show_command_palette: bool,
    /// Currently selected index within the command palette.
    pub command_palette_index: usize,
    /// Whether the shortcut hints bar is visible (triggered by `?` on empty line).
    pub show_hints: bool,
    /// Set of thread indices whose branches are expanded in the list.
    /// Active threads are always expanded; this tracks user toggles.
    pub expanded: HashSet<usize>,
    /// Recent notes (captures) for the active thread/branch, shown in the status pane.
    pub recent_notes: Vec<Capture>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Insert,
            threads: Vec::new(),
            selected: SelectedItem::Thread(0),
            last_reply: None,
            error_message: None,
            scope_context: ScopeContext::default(),
            poll_watermark: None,
            should_quit: false,
            show_command_palette: false,
            command_palette_index: 0,
            show_hints: false,
            expanded: HashSet::new(),
            recent_notes: Vec::new(),
        }
    }

    /// Build a flat list of all visible (selectable) rows in the thread list.
    /// Each entry is a `SelectedItem` — either a thread or a branch within an expanded thread.
    pub fn visible_rows(&self) -> Vec<SelectedItem> {
        let mut rows = Vec::new();
        for (i, entry) in self.threads.iter().enumerate() {
            rows.push(SelectedItem::Thread(i));
            if self.is_expanded(i) {
                for (j, _branch) in entry.branches.iter().enumerate() {
                    rows.push(SelectedItem::Branch(i, j));
                }
            }
        }
        rows
    }

    /// Refresh state from the database.
    pub fn refresh_from_db(&mut self, conn: &Connection) {
        // Load active and paused threads
        let threads =
            thread_repo::list_by_statuses(conn, &[ThreadStatus::Active, ThreadStatus::Paused])
                .unwrap_or_default();

        self.threads = threads
            .into_iter()
            .map(|thread| {
                let branches = branch_repo::find_by_thread(conn, &thread.id).unwrap_or_default();
                ThreadEntry { thread, branches }
            })
            .collect();

        // Load scope context for the active thread
        self.scope_context = ScopeContext::default();
        if let Some(active) = self.active_thread() {
            let scopes =
                scope_repo::find_by_target(conn, "thread", &active.thread.id).unwrap_or_default();
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

        // Load recent notes for the active thread (and its active branch)
        self.recent_notes = Vec::new();
        if let Some(active) = self.active_thread() {
            // Notes on the thread itself
            let thread_captures =
                capture_repo::find_by_target(conn, "thread", &active.thread.id, 5)
                    .unwrap_or_default();

            // Notes on active branches
            let branch_captures: Vec<Capture> = active
                .branches
                .iter()
                .filter(|b| b.status == liminal_flow_core::model::BranchStatus::Active)
                .flat_map(|b| {
                    capture_repo::find_by_target(conn, "branch", &b.id, 3).unwrap_or_default()
                })
                .collect();

            // Merge and filter to just notes (AddNote intent), most recent first
            let mut all: Vec<Capture> = thread_captures
                .into_iter()
                .chain(branch_captures)
                .filter(|c| {
                    c.inferred_intent
                        .as_ref()
                        .is_some_and(|i| *i == liminal_flow_core::model::Intent::AddNote)
                })
                .collect();
            all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            all.truncate(5);
            self.recent_notes = all;
        }

        // Clamp selection to valid visible rows
        self.clamp_selection();
    }

    /// Return the active thread entry, if any.
    pub fn active_thread(&self) -> Option<&ThreadEntry> {
        self.threads
            .iter()
            .find(|e| e.thread.status == ThreadStatus::Active)
    }

    /// Move selection to the next visible row, wrapping around.
    pub fn select_next(&mut self) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }
        let current = rows.iter().position(|r| *r == self.selected).unwrap_or(0);
        let next = (current + 1) % rows.len();
        self.selected = rows[next].clone();
    }

    /// Move selection to the previous visible row, wrapping around.
    pub fn select_prev(&mut self) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }
        let current = rows.iter().position(|r| *r == self.selected).unwrap_or(0);
        let prev = if current == 0 {
            rows.len() - 1
        } else {
            current - 1
        };
        self.selected = rows[prev].clone();
    }

    /// Toggle expansion of the currently selected thread's branches.
    /// If a branch is selected, toggles the parent thread.
    pub fn toggle_expanded(&mut self) {
        let thread_idx = match &self.selected {
            SelectedItem::Thread(i) => *i,
            SelectedItem::Branch(i, _) => *i,
        };
        if self.expanded.contains(&thread_idx) {
            self.expanded.remove(&thread_idx);
            // If a branch was selected, move selection to the parent thread
            if matches!(self.selected, SelectedItem::Branch(_, _)) {
                self.selected = SelectedItem::Thread(thread_idx);
            }
        } else {
            self.expanded.insert(thread_idx);
        }
    }

    /// Whether a thread at the given index should show its branches.
    /// Active threads are always expanded.
    pub fn is_expanded(&self, index: usize) -> bool {
        if let Some(entry) = self.threads.get(index) {
            if entry.thread.status == ThreadStatus::Active {
                return true;
            }
        }
        self.expanded.contains(&index)
    }

    /// Return the thread index of the currently selected item.
    pub fn selected_thread_index(&self) -> usize {
        match &self.selected {
            SelectedItem::Thread(i) => *i,
            SelectedItem::Branch(i, _) => *i,
        }
    }

    /// Return the FlowId of the selected item (thread ID or branch ID).
    pub fn selected_id(&self) -> Option<FlowId> {
        match &self.selected {
            SelectedItem::Thread(i) => self.threads.get(*i).map(|e| e.thread.id.clone()),
            SelectedItem::Branch(i, j) => self
                .threads
                .get(*i)
                .and_then(|e| e.branches.get(*j))
                .map(|b| b.id.clone()),
        }
    }

    /// Ensure selection points to a valid visible row.
    fn clamp_selection(&mut self) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            self.selected = SelectedItem::Thread(0);
            return;
        }
        if !rows.contains(&self.selected) {
            // Try to stay on the same thread
            let thread_idx = self.selected_thread_index();
            if rows.contains(&SelectedItem::Thread(thread_idx)) {
                self.selected = SelectedItem::Thread(thread_idx);
            } else {
                self.selected = rows[0].clone();
            }
        }
    }
}
