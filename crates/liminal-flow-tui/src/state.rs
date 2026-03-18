// TUI state management
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use chrono::Utc;
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

pub struct SlashCommand {
    pub syntax: &'static str,
    pub description: &'static str,
    pub requires_argument: bool,
}

impl SlashCommand {
    pub fn name(&self) -> &'static str {
        self.syntax.split_whitespace().next().unwrap_or(self.syntax)
    }
}

/// Slash commands available in the command palette.
pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        syntax: "/now <text>",
        description: "Set or replace the current thread",
        requires_argument: true,
    },
    SlashCommand {
        syntax: "/branch <text>",
        description: "Start a branch beneath current thread",
        requires_argument: true,
    },
    SlashCommand {
        syntax: "/back",
        description: "Return from the active branch to the parent thread",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/park",
        description: "Park the selected branch",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/archive",
        description: "Archive the selected item",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/note <note>",
        description: "Attach a note to the selected item",
        requires_argument: true,
    },
    SlashCommand {
        syntax: "/where",
        description: "Show current thread and branches",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/resume",
        description: "Resume the selected item",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/pause",
        description: "Pause the selected thread",
        requires_argument: false,
    },
    SlashCommand {
        syntax: "/done",
        description: "Mark the selected item done",
        requires_argument: false,
    },
];

/// Return the active slash-command token from palette input.
fn command_palette_query(query: &str) -> &str {
    query.split_whitespace().next().unwrap_or("")
}

fn slash_command_by_name(name: &str) -> Option<&'static SlashCommand> {
    // Use exact (case-sensitive) matching to stay consistent with the core
    // parser, which only recognises lowercase command names.
    SLASH_COMMANDS
        .iter()
        .find(|command| command.name().trim_start_matches('/') == name)
}

pub fn should_keep_command_palette_open(query: &str) -> bool {
    let normalized = query.trim_start();
    let Some(rest) = normalized.strip_prefix('/') else {
        return false;
    };

    if rest.trim().is_empty() {
        return true;
    }

    let command_name = rest.split_whitespace().next().unwrap_or("");
    let has_trailing_text = rest[command_name.len()..]
        .chars()
        .next()
        .is_some_and(char::is_whitespace);

    match slash_command_by_name(command_name) {
        Some(_) if has_trailing_text => false,
        Some(command) => command.requires_argument,
        // Unknown prefix: keep palette open so the user can correct/select a
        // command, even if there is trailing text (the argument they already typed).
        None => true,
    }
}

/// Return slash commands filtered by the current palette query.
pub fn filtered_slash_commands(query: &str) -> Vec<(usize, &'static str, &'static str)> {
    let normalized = command_palette_query(query).trim();
    let needle = normalized
        .strip_prefix('/')
        .unwrap_or(normalized)
        .trim()
        .to_ascii_lowercase();

    let mut matches = SLASH_COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, command)| {
            if needle.is_empty() {
                return true;
            }

            let cmd_name = command.name().trim_start_matches('/').to_ascii_lowercase();
            let desc_text = command.description.to_ascii_lowercase();

            cmd_name.contains(&needle) || desc_text.contains(&needle)
        })
        .map(|(index, command)| (index, command.syntax, command.description))
        .collect::<Vec<_>>();

    matches.sort_by_key(|(_index, cmd, desc)| {
        if needle.is_empty() {
            return (0_u8, 0_usize, 0_usize);
        }

        let cmd_name = cmd
            .split_whitespace()
            .next()
            .unwrap_or(cmd)
            .trim_start_matches('/')
            .to_ascii_lowercase();
        let desc_text = desc.to_ascii_lowercase();

        if let Some(position) = cmd_name.find(&needle) {
            (0_u8, position, 0_usize)
        } else if let Some(position) = desc_text.find(&needle) {
            (1_u8, position, 0_usize)
        } else {
            (2_u8, usize::MAX, 0_usize)
        }
    });

    matches
}

/// Keyboard shortcut hints shown when ? is typed on an empty line.
pub const SHORTCUT_HINTS: &[(&str, &str)] = &[
    ("/ for commands (Insert)", "Esc to Normal mode"),
    ("Enter submits/expands (Insert)", "i switches to Insert"),
    ("Up/Down move selection", "r resumes selected (Normal)"),
    (
        "p parks selected branch (Normal)",
        "d marks selected done (Normal)",
    ),
    ("A archives selected item (Normal)", "q quits (Normal)"),
    (
        "Plain text targets active item",
        "r revives selected done item",
    ),
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
    pub selected_scope_context: ScopeContext,
    pub status_scroll: u16,
    pub thread_list_scroll: u16,
    pub poll_watermark: Option<String>,
    pub should_quit: bool,
    /// Whether the command palette popup is visible (triggered by `/` on empty line).
    pub show_command_palette: bool,
    /// Currently selected index within the command palette.
    pub command_palette_index: usize,
    /// Whether the shortcut hints bar is visible (triggered by `?` on empty line).
    pub show_hints: bool,
    pub help_scroll: u16,
    /// Set of thread indices whose branches are expanded in the list.
    /// Active threads are always expanded; this tracks user toggles.
    pub expanded: HashSet<usize>,
    /// Recent notes (captures) for the selected thread or branch, shown in the status pane.
    pub selected_notes: Vec<Capture>,
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
            selected_scope_context: ScopeContext::default(),
            status_scroll: 0,
            thread_list_scroll: 0,
            poll_watermark: None,
            should_quit: false,
            show_command_palette: false,
            command_palette_index: 0,
            show_hints: false,
            help_scroll: 0,
            expanded: HashSet::new(),
            selected_notes: Vec::new(),
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
        let now = Utc::now().to_rfc3339();
        let _ = thread_repo::normalize_active(conn, &now);

        // Load working threads, including done tombstones until they are archived.
        let threads = thread_repo::list_by_statuses(
            conn,
            &[
                ThreadStatus::Active,
                ThreadStatus::Paused,
                ThreadStatus::Done,
            ],
        )
        .unwrap_or_default();

        self.threads = threads
            .into_iter()
            .map(|thread| {
                let _ = branch_repo::normalize_active_for_thread(conn, &thread.id, &now);
                let branches =
                    branch_repo::find_visible_by_thread(conn, &thread.id).unwrap_or_default();
                ThreadEntry { thread, branches }
            })
            .collect();

        // Clamp selection to valid visible rows
        self.clamp_selection();
        self.refresh_selected_details(conn);
    }

    /// Return the active thread entry, if any.
    pub fn active_thread(&self) -> Option<&ThreadEntry> {
        self.threads
            .iter()
            .find(|e| e.thread.status == ThreadStatus::Active)
    }

    /// Return the active branch on the active thread, if any.
    pub fn active_branch(&self) -> Option<&Branch> {
        self.active_thread().and_then(|entry| {
            entry
                .branches
                .iter()
                .find(|b| b.status == liminal_flow_core::model::BranchStatus::Active)
        })
    }

    /// Return the display label for the current capture target.
    pub fn active_capture_target_label(&self) -> Option<String> {
        if let Some(branch) = self.active_branch() {
            return Some(format!("branch: {}", branch.title));
        }

        self.active_thread()
            .map(|entry| format!("thread: {}", entry.thread.title))
    }

    /// Return the currently selected thread entry.
    pub fn selected_thread(&self) -> Option<&ThreadEntry> {
        match &self.selected {
            SelectedItem::Thread(i) | SelectedItem::Branch(i, _) => self.threads.get(*i),
        }
    }

    /// Return the currently selected branch, if any.
    pub fn selected_branch(&self) -> Option<&Branch> {
        match &self.selected {
            SelectedItem::Thread(_) => None,
            SelectedItem::Branch(i, j) => self.threads.get(*i).and_then(|e| e.branches.get(*j)),
        }
    }

    /// Return a label describing the selected item for the status pane.
    pub fn selected_title(&self) -> Option<String> {
        match (
            &self.selected,
            self.selected_thread(),
            self.selected_branch(),
        ) {
            (SelectedItem::Thread(_), Some(entry), _) => Some(entry.thread.title.clone()),
            (SelectedItem::Branch(_, _), _, Some(branch)) => Some(branch.title.clone()),
            _ => None,
        }
    }

    /// Return whether the selected item is currently active.
    pub fn selected_is_active(&self) -> bool {
        match (
            &self.selected,
            self.selected_thread(),
            self.selected_branch(),
        ) {
            (SelectedItem::Thread(_), Some(entry), _) => {
                entry.thread.status == ThreadStatus::Active
            }
            (SelectedItem::Branch(_, _), Some(entry), Some(branch)) => {
                entry.thread.status == ThreadStatus::Active
                    && branch.status == liminal_flow_core::model::BranchStatus::Active
            }
            _ => false,
        }
    }

    /// Return a short status label for the selected item.
    pub fn selected_status_label(&self) -> Option<String> {
        match (
            &self.selected,
            self.selected_thread(),
            self.selected_branch(),
        ) {
            (SelectedItem::Thread(_), Some(entry), _) => Some(entry.thread.status.to_string()),
            (SelectedItem::Branch(_, _), Some(entry), Some(branch)) => {
                match (entry.thread.status.clone(), branch.status.clone()) {
                    (ThreadStatus::Paused, liminal_flow_core::model::BranchStatus::Active) => {
                        Some("inactive (thread paused)".into())
                    }
                    (ThreadStatus::Done, liminal_flow_core::model::BranchStatus::Active) => {
                        Some("inactive (thread done)".into())
                    }
                    _ => Some(branch.status.to_string()),
                }
            }
            _ => None,
        }
    }

    /// Return a compact label describing the selected item kind.
    pub fn selected_kind_label(&self) -> &'static str {
        match self.selected {
            SelectedItem::Thread(_) => "Thread",
            SelectedItem::Branch(_, _) => "Branch",
        }
    }

    /// Return the parent thread title when a branch is selected.
    pub fn selected_parent_title(&self) -> Option<String> {
        match (&self.selected, self.selected_thread()) {
            (SelectedItem::Branch(_, _), Some(entry)) => Some(entry.thread.title.clone()),
            _ => None,
        }
    }

    /// Move selection to the currently active branch, or the active thread if no branch is active.
    pub fn select_active_item(&mut self) {
        for (thread_idx, entry) in self.threads.iter().enumerate() {
            if entry.thread.status != ThreadStatus::Active {
                continue;
            }

            self.expanded.insert(thread_idx);

            if let Some(branch_idx) = entry
                .branches
                .iter()
                .position(|branch| branch.status == liminal_flow_core::model::BranchStatus::Active)
            {
                self.selected = SelectedItem::Branch(thread_idx, branch_idx);
                self.status_scroll = 0;
            } else {
                self.selected = SelectedItem::Thread(thread_idx);
                self.status_scroll = 0;
            }
            return;
        }
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
        self.status_scroll = 0;
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
        self.status_scroll = 0;
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
                self.status_scroll = 0;
            }
        } else {
            self.expanded.insert(thread_idx);
        }
    }

    /// Whether a thread at the given index should show its branches.
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.contains(&index)
    }

    /// Return the thread index of the currently selected item.
    pub fn selected_thread_index(&self) -> usize {
        match &self.selected {
            SelectedItem::Thread(i) => *i,
            SelectedItem::Branch(i, _) => *i,
        }
    }

    /// Keep the selected row visible within the thread-list viewport.
    pub fn ensure_thread_selection_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            self.thread_list_scroll = 0;
            return;
        }

        let rows = self.visible_rows();
        let Some(selected_row) = rows.iter().position(|row| *row == self.selected) else {
            self.thread_list_scroll = 0;
            return;
        };

        let scroll = self.thread_list_scroll as usize;
        if selected_row < scroll {
            self.thread_list_scroll = selected_row as u16;
            return;
        }

        let viewport_end = scroll + viewport_height;
        if selected_row >= viewport_end {
            self.thread_list_scroll =
                selected_row.saturating_sub(viewport_height.saturating_sub(1)) as u16;
        }
    }

    /// Clamp the thread-list scroll offset to the current number of visible rows.
    pub fn clamp_thread_list_scroll(&mut self, viewport_height: usize) {
        let rows = self.visible_rows();
        let max_scroll = rows.len().saturating_sub(viewport_height) as u16;
        self.thread_list_scroll = self.thread_list_scroll.min(max_scroll);
    }

    /// Scroll the thread list viewport by a signed delta.
    pub fn scroll_thread_list(&mut self, delta: i16, viewport_height: usize) {
        let next = if delta.is_negative() {
            self.thread_list_scroll.saturating_sub(delta.unsigned_abs())
        } else {
            self.thread_list_scroll.saturating_add(delta as u16)
        };
        self.thread_list_scroll = next;
        self.clamp_thread_list_scroll(viewport_height);
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
                self.status_scroll = 0;
            } else {
                self.selected = rows[0].clone();
                self.status_scroll = 0;
            }
        }
    }

    pub fn refresh_selected_details(&mut self, conn: &Connection) {
        self.selected_scope_context = ScopeContext::default();
        self.selected_notes = Vec::new();

        let (target_type, target_id) = match (
            &self.selected,
            self.selected_thread(),
            self.selected_branch(),
        ) {
            (SelectedItem::Thread(_), Some(entry), _) => ("thread", entry.thread.id.clone()),
            (SelectedItem::Branch(_, _), _, Some(branch)) => ("branch", branch.id.clone()),
            _ => return,
        };

        let scopes = scope_repo::find_by_target(conn, target_type, &target_id).unwrap_or_default();
        for scope in &scopes {
            match scope.kind {
                liminal_flow_core::model::ScopeKind::Repo => {
                    self.selected_scope_context.repo = Some(scope.value.clone());
                }
                liminal_flow_core::model::ScopeKind::GitBranch => {
                    self.selected_scope_context.git_branch = Some(scope.value.clone());
                }
                liminal_flow_core::model::ScopeKind::Cwd => {
                    self.selected_scope_context.cwd = Some(scope.value.clone());
                }
                _ => {}
            }
        }

        let mut notes = capture_repo::find_by_target(conn, target_type, &target_id, 5)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| {
                c.inferred_intent
                    .as_ref()
                    .is_some_and(|i| *i == liminal_flow_core::model::Intent::AddNote)
            })
            .collect::<Vec<_>>();
        notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        notes.truncate(5);
        self.selected_notes = notes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liminal_flow_core::model::{BranchStatus, CaptureSource, Intent, ThreadStatus};
    use liminal_flow_store::db::open_store_in_memory;
    use liminal_flow_store::repo::{branch_repo, capture_repo, thread_repo};

    #[test]
    fn selected_notes_follow_the_selected_branch() {
        let conn = open_store_in_memory().unwrap();
        let now = Utc::now();

        let thread = Thread {
            id: FlowId::from("t1"),
            title: "thread".into(),
            raw_origin_text: "thread".into(),
            status: ThreadStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        };
        thread_repo::upsert(&conn, &thread).unwrap();

        let branch_one = Branch {
            id: FlowId::from("b1"),
            thread_id: thread.id.clone(),
            title: "branch one".into(),
            status: BranchStatus::Active,
            short_summary: None,
            created_at: now,
            updated_at: now,
        };
        let branch_two = Branch {
            id: FlowId::from("b2"),
            thread_id: thread.id.clone(),
            title: "branch two".into(),
            status: BranchStatus::Parked,
            short_summary: None,
            created_at: now,
            updated_at: now,
        };
        branch_repo::upsert(&conn, &branch_one).unwrap();
        branch_repo::upsert(&conn, &branch_two).unwrap();

        capture_repo::insert(
            &conn,
            &Capture {
                id: FlowId::from("c1"),
                target_type: "branch".into(),
                target_id: FlowId::from("b1"),
                text: "note on branch one".into(),
                source: CaptureSource::Keyboard,
                inferred_intent: Some(Intent::AddNote),
                created_at: now,
            },
        )
        .unwrap();
        capture_repo::insert(
            &conn,
            &Capture {
                id: FlowId::from("c2"),
                target_type: "branch".into(),
                target_id: FlowId::from("b2"),
                text: "note on branch two".into(),
                source: CaptureSource::Keyboard,
                inferred_intent: Some(Intent::AddNote),
                created_at: now,
            },
        )
        .unwrap();

        let mut state = TuiState::new();
        state.refresh_from_db(&conn);

        state.selected = SelectedItem::Branch(0, 0);
        state.refresh_selected_details(&conn);
        assert_eq!(state.selected_notes.len(), 1);
        assert_eq!(state.selected_notes[0].text, "note on branch one");

        state.selected = SelectedItem::Branch(0, 1);
        state.refresh_selected_details(&conn);
        assert_eq!(state.selected_notes.len(), 1);
        assert_eq!(state.selected_notes[0].text, "note on branch two");
    }

    #[test]
    fn slash_commands_filter_by_query() {
        let filtered = filtered_slash_commands("/res");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1, "/resume");

        let filtered = filtered_slash_commands("/note");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1, "/note <note>");

        let filtered = filtered_slash_commands("/arch");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1, "/archive");
    }

    #[test]
    fn slash_command_name_matches_outrank_description_matches() {
        let filtered = filtered_slash_commands("/par");
        assert_eq!(filtered[0].1, "/park");
        assert_eq!(filtered[1].1, "/back");
    }

    #[test]
    fn command_palette_query_ignores_trailing_text() {
        assert_eq!(command_palette_query("/no cat"), "/no");
        assert_eq!(command_palette_query("/now improve"), "/now");
        assert_eq!(command_palette_query("   /done shipped"), "/done");
    }

    #[test]
    fn slash_command_filter_ignores_trailing_text_after_token() {
        let filtered = filtered_slash_commands("/no cat");
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|(_, cmd, _)| *cmd == "/now <text>"));
        assert!(filtered.iter().any(|(_, cmd, _)| *cmd == "/note <note>"));
    }

    #[test]
    fn palette_stays_open_for_partial_and_argument_required_commands() {
        assert!(should_keep_command_palette_open("/"));
        assert!(should_keep_command_palette_open("/no"));
        assert!(should_keep_command_palette_open("/now"));
        assert!(should_keep_command_palette_open("/branch"));
        assert!(should_keep_command_palette_open("/note"));
    }

    #[test]
    fn palette_closes_for_complete_commands_and_argument_entry() {
        assert!(!should_keep_command_palette_open("/where"));
        assert!(!should_keep_command_palette_open("/resume"));
        assert!(!should_keep_command_palette_open("/archive"));
        assert!(!should_keep_command_palette_open("/now test thread"));
        assert!(!should_keep_command_palette_open(
            "/pause blocked on review"
        ));
    }

    #[test]
    fn palette_stays_open_for_unknown_command_with_trailing_text() {
        // Unknown prefix with trailing text: the user is likely editing the
        // command name to pick a different one (e.g. /branch → /bran → /now),
        // so keep the palette open for correction.
        assert!(should_keep_command_palette_open("/bran meow"));
        assert!(should_keep_command_palette_open("/foo bar"));
    }

    #[test]
    fn palette_stays_open_for_mixed_case_commands() {
        // Mixed-case commands are not recognised by the parser, so the palette
        // should stay open to let the user correct/complete them.
        assert!(should_keep_command_palette_open("/WHERE"));
        assert!(should_keep_command_palette_open("/Resume"));
        assert!(should_keep_command_palette_open("/DONE"));
    }
}
