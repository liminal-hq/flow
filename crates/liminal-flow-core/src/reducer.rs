// State reducer — applies events to produce new application state
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use crate::error::CoreError;
use crate::event::AppEvent;
use crate::model::{Branch, BranchStatus, Thread, ThreadStatus};
use crate::state::AppState;

/// Apply an event to the current state, returning the new state.
///
/// This is a pure function — it does not perform I/O or side effects.
/// The five core rules are enforced here:
///
/// 1. Only one current thread at a time
/// 2. Branches belong to one thread
/// 3. Returning to parent parks active branches
/// 4. Raw capture is always stored (handled by the caller)
/// 5. Queries do not mutate work state
pub fn apply(state: &mut AppState, event: &AppEvent) -> Result<(), CoreError> {
    match event {
        AppEvent::ThreadSetCurrent {
            thread_id,
            title,
            raw_text,
            created_at,
        } => {
            // Rule 1: pause any currently active thread
            if let Some(ref current_id) = state.current_thread_id {
                if let Some(current_thread) = state.threads.get_mut(current_id) {
                    if current_thread.status == ThreadStatus::Active {
                        current_thread.status = ThreadStatus::Paused;
                        current_thread.updated_at = *created_at;
                    }
                }
            }

            // Create the new thread
            let thread = Thread {
                id: thread_id.clone(),
                title: title.clone(),
                raw_origin_text: raw_text.clone(),
                status: ThreadStatus::Active,
                short_summary: None,
                created_at: *created_at,
                updated_at: *created_at,
            };

            state.threads.insert(thread_id.clone(), thread);
            state.current_thread_id = Some(thread_id.clone());
            state.last_reply = Some(format!("Current thread: {title}"));
        }

        AppEvent::BranchStarted {
            branch_id,
            thread_id,
            title,
            created_at,
        } => {
            // Rule 2: branch belongs to the specified thread
            if !state.threads.contains_key(thread_id) {
                return Err(CoreError::ThreadNotFound(thread_id.to_string()));
            }

            let branch = Branch {
                id: branch_id.clone(),
                thread_id: thread_id.clone(),
                title: title.clone(),
                status: BranchStatus::Active,
                short_summary: None,
                created_at: *created_at,
                updated_at: *created_at,
            };

            state.branches.insert(branch_id.clone(), branch);
            state.last_reply = Some(format!("Added branch: {title}"));
        }

        AppEvent::ReturnedToParent {
            thread_id,
            parked_branch_ids,
            created_at,
        } => {
            // Rule 3: park all active branches for this thread
            for branch_id in parked_branch_ids {
                if let Some(branch) = state.branches.get_mut(branch_id) {
                    branch.status = BranchStatus::Parked;
                    branch.updated_at = *created_at;
                }
            }

            // Ensure the thread is the current focus
            state.current_thread_id = Some(thread_id.clone());

            let thread_title = state
                .threads
                .get(thread_id)
                .map(|t| t.title.clone())
                .unwrap_or_else(|| "unknown".to_string());

            state.last_reply = Some(format!("Returned to parent thread: {thread_title}"));
        }

        AppEvent::NoteAttached { .. } => {
            state.last_reply = Some("Note attached.".to_string());
        }

        AppEvent::ThreadPaused {
            thread_id,
            created_at,
        } => {
            if let Some(thread) = state.threads.get_mut(thread_id) {
                thread.status = ThreadStatus::Paused;
                thread.updated_at = *created_at;

                let title = thread.title.clone();
                state.last_reply = Some(format!("Paused thread: {title}"));

                // Clear current thread if it was the paused one
                if state.current_thread_id.as_ref() == Some(thread_id) {
                    state.current_thread_id = None;
                }
            } else {
                return Err(CoreError::ThreadNotFound(thread_id.to_string()));
            }
        }

        AppEvent::ThreadMarkedDone {
            thread_id,
            created_at,
        } => {
            if let Some(thread) = state.threads.get_mut(thread_id) {
                thread.status = ThreadStatus::Done;
                thread.updated_at = *created_at;

                let title = thread.title.clone();
                state.last_reply = Some(format!("Done: {title}"));

                // Clear current thread if it was the completed one
                if state.current_thread_id.as_ref() == Some(thread_id) {
                    state.current_thread_id = None;
                }
            } else {
                return Err(CoreError::ThreadNotFound(thread_id.to_string()));
            }
        }

        AppEvent::CaptureReceived { .. } => {
            // Rule 4: raw capture is always stored (persistence handled by caller)
            // No state mutation needed here.
        }

        AppEvent::ScopeObserved { .. } => {
            // Scope attachment is handled by the store layer
        }

        AppEvent::ReplyUpdated { text, .. } => {
            state.last_reply = Some(text.clone());
        }
    }

    Ok(())
}

/// Generate a reply describing the current working state (for `/where` and queries).
///
/// Rule 5: queries do not mutate work state.
pub fn query_current(state: &AppState) -> String {
    let Some(ref thread_id) = state.current_thread_id else {
        return "(no active thread)".to_string();
    };

    let Some(thread) = state.threads.get(thread_id) else {
        return "(no active thread)".to_string();
    };

    let mut reply = format!("Current thread: {}", thread.title);

    // Collect branches for this thread
    let mut branches: Vec<&Branch> = state
        .branches
        .values()
        .filter(|b| b.thread_id == *thread_id)
        .collect();

    // Sort by creation time for deterministic output
    branches.sort_by_key(|b| b.created_at);

    if !branches.is_empty() {
        let branch_descriptions: Vec<String> = branches
            .iter()
            .map(|b| {
                if b.status == BranchStatus::Active {
                    b.title.clone()
                } else {
                    format!("{} ({})", b.title, b.status)
                }
            })
            .collect();

        reply.push_str(&format!("\nBranches: {}", branch_descriptions.join(", ")));
    }

    reply
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FlowId;
    use chrono::Utc;

    fn now() -> chrono::DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn set_current_thread_on_empty_state() {
        let mut state = AppState::default();
        let event = AppEvent::ThreadSetCurrent {
            thread_id: FlowId::from("t1"),
            title: "improving AIDX".into(),
            raw_text: "I'm improving AIDX for the component library".into(),
            created_at: now(),
        };

        apply(&mut state, &event).unwrap();

        assert_eq!(state.current_thread_id, Some(FlowId::from("t1")));
        assert_eq!(state.threads.len(), 1);
        assert_eq!(
            state.threads.get(&FlowId::from("t1")).unwrap().title,
            "improving AIDX"
        );
        assert_eq!(
            state.threads.get(&FlowId::from("t1")).unwrap().status,
            ThreadStatus::Active
        );
        assert_eq!(
            state.last_reply,
            Some("Current thread: improving AIDX".into())
        );
    }

    #[test]
    fn setting_new_thread_pauses_previous() {
        let mut state = AppState::default();

        let e1 = AppEvent::ThreadSetCurrent {
            thread_id: FlowId::from("t1"),
            title: "improving AIDX".into(),
            raw_text: "improving AIDX".into(),
            created_at: now(),
        };
        apply(&mut state, &e1).unwrap();

        let e2 = AppEvent::ThreadSetCurrent {
            thread_id: FlowId::from("t2"),
            title: "debugging sync".into(),
            raw_text: "debugging sync".into(),
            created_at: now(),
        };
        apply(&mut state, &e2).unwrap();

        assert_eq!(state.current_thread_id, Some(FlowId::from("t2")));
        assert_eq!(
            state.threads.get(&FlowId::from("t1")).unwrap().status,
            ThreadStatus::Paused
        );
        assert_eq!(
            state.threads.get(&FlowId::from("t2")).unwrap().status,
            ThreadStatus::Active
        );
    }

    #[test]
    fn branch_started_under_thread() {
        let mut state = AppState::default();
        let ts = now();

        apply(
            &mut state,
            &AppEvent::ThreadSetCurrent {
                thread_id: FlowId::from("t1"),
                title: "improving AIDX".into(),
                raw_text: "improving AIDX".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::BranchStarted {
                branch_id: FlowId::from("b1"),
                thread_id: FlowId::from("t1"),
                title: "answering support".into(),
                created_at: ts,
            },
        )
        .unwrap();

        assert_eq!(state.branches.len(), 1);
        let branch = state.branches.get(&FlowId::from("b1")).unwrap();
        assert_eq!(branch.thread_id, FlowId::from("t1"));
        assert_eq!(branch.status, BranchStatus::Active);
        assert_eq!(
            state.last_reply,
            Some("Added branch: answering support".into())
        );
    }

    #[test]
    fn branch_on_missing_thread_fails() {
        let mut state = AppState::default();
        let result = apply(
            &mut state,
            &AppEvent::BranchStarted {
                branch_id: FlowId::from("b1"),
                thread_id: FlowId::from("nonexistent"),
                title: "orphan branch".into(),
                created_at: now(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn return_to_parent_parks_branches() {
        let mut state = AppState::default();
        let ts = now();

        apply(
            &mut state,
            &AppEvent::ThreadSetCurrent {
                thread_id: FlowId::from("t1"),
                title: "improving AIDX".into(),
                raw_text: "improving AIDX".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::BranchStarted {
                branch_id: FlowId::from("b1"),
                thread_id: FlowId::from("t1"),
                title: "answering support".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::BranchStarted {
                branch_id: FlowId::from("b2"),
                thread_id: FlowId::from("t1"),
                title: "reading article".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::ReturnedToParent {
                thread_id: FlowId::from("t1"),
                parked_branch_ids: vec![FlowId::from("b1"), FlowId::from("b2")],
                created_at: ts,
            },
        )
        .unwrap();

        assert_eq!(
            state.branches.get(&FlowId::from("b1")).unwrap().status,
            BranchStatus::Parked
        );
        assert_eq!(
            state.branches.get(&FlowId::from("b2")).unwrap().status,
            BranchStatus::Parked
        );
        assert_eq!(state.current_thread_id, Some(FlowId::from("t1")));
        assert!(state
            .last_reply
            .as_ref()
            .unwrap()
            .contains("Returned to parent"));
    }

    #[test]
    fn pause_thread() {
        let mut state = AppState::default();
        let ts = now();

        apply(
            &mut state,
            &AppEvent::ThreadSetCurrent {
                thread_id: FlowId::from("t1"),
                title: "improving AIDX".into(),
                raw_text: "improving AIDX".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::ThreadPaused {
                thread_id: FlowId::from("t1"),
                created_at: ts,
            },
        )
        .unwrap();

        assert_eq!(
            state.threads.get(&FlowId::from("t1")).unwrap().status,
            ThreadStatus::Paused
        );
        assert_eq!(state.current_thread_id, None);
    }

    #[test]
    fn mark_thread_done() {
        let mut state = AppState::default();
        let ts = now();

        apply(
            &mut state,
            &AppEvent::ThreadSetCurrent {
                thread_id: FlowId::from("t1"),
                title: "improving AIDX".into(),
                raw_text: "improving AIDX".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::ThreadMarkedDone {
                thread_id: FlowId::from("t1"),
                created_at: ts,
            },
        )
        .unwrap();

        assert_eq!(
            state.threads.get(&FlowId::from("t1")).unwrap().status,
            ThreadStatus::Done
        );
        assert_eq!(state.current_thread_id, None);
        assert!(state.last_reply.as_ref().unwrap().contains("Done"));
    }

    #[test]
    fn query_current_with_no_thread() {
        let state = AppState::default();
        let reply = query_current(&state);
        assert_eq!(reply, "(no active thread)");
    }

    #[test]
    fn query_current_with_thread_and_branches() {
        let mut state = AppState::default();
        let ts = now();

        apply(
            &mut state,
            &AppEvent::ThreadSetCurrent {
                thread_id: FlowId::from("t1"),
                title: "improving AIDX".into(),
                raw_text: "improving AIDX".into(),
                created_at: ts,
            },
        )
        .unwrap();

        apply(
            &mut state,
            &AppEvent::BranchStarted {
                branch_id: FlowId::from("b1"),
                thread_id: FlowId::from("t1"),
                title: "answering support".into(),
                created_at: ts,
            },
        )
        .unwrap();

        let reply = query_current(&state);
        assert!(reply.contains("Current thread: improving AIDX"));
        assert!(reply.contains("answering support"));
    }
}
