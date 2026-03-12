// Domain events for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{CaptureSource, FlowId};

/// All domain events that can occur in Liminal Flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppEvent {
    CaptureReceived {
        capture_id: FlowId,
        text: String,
        source: CaptureSource,
        created_at: DateTime<Utc>,
    },
    ThreadSetCurrent {
        thread_id: FlowId,
        title: String,
        raw_text: String,
        created_at: DateTime<Utc>,
    },
    BranchStarted {
        branch_id: FlowId,
        thread_id: FlowId,
        title: String,
        created_at: DateTime<Utc>,
    },
    ReturnedToParent {
        thread_id: FlowId,
        parked_branch_ids: Vec<FlowId>,
        created_at: DateTime<Utc>,
    },
    BranchParked {
        branch_id: FlowId,
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    NoteAttached {
        capture_id: FlowId,
        target_type: String,
        target_id: FlowId,
        created_at: DateTime<Utc>,
    },
    ThreadPaused {
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    ThreadMarkedDone {
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    BranchMarkedDone {
        branch_id: FlowId,
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    ThreadArchived {
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    BranchArchived {
        branch_id: FlowId,
        thread_id: FlowId,
        created_at: DateTime<Utc>,
    },
    ScopeObserved {
        scope_id: FlowId,
        target_type: String,
        target_id: FlowId,
        kind: String,
        value: String,
        confidence: f64,
        created_at: DateTime<Utc>,
    },
    ReplyUpdated {
        text: String,
        created_at: DateTime<Utc>,
    },
}

impl AppEvent {
    /// Return the event type name as a string.
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::CaptureReceived { .. } => "capture_received",
            Self::ThreadSetCurrent { .. } => "thread_set_current",
            Self::BranchStarted { .. } => "branch_started",
            Self::ReturnedToParent { .. } => "returned_to_parent",
            Self::BranchParked { .. } => "branch_parked",
            Self::NoteAttached { .. } => "note_attached",
            Self::ThreadPaused { .. } => "thread_paused",
            Self::ThreadMarkedDone { .. } => "thread_marked_done",
            Self::BranchMarkedDone { .. } => "branch_marked_done",
            Self::ThreadArchived { .. } => "thread_archived",
            Self::BranchArchived { .. } => "branch_archived",
            Self::ScopeObserved { .. } => "scope_observed",
            Self::ReplyUpdated { .. } => "reply_updated",
        }
    }
}
