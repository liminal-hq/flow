// Application state for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::model::{Branch, FlowId, Thread};

/// The in-memory working state of the application.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// The ID of the currently active thread, if any.
    pub current_thread_id: Option<FlowId>,

    /// All known threads, keyed by ID.
    pub threads: HashMap<FlowId, Thread>,

    /// All known branches, keyed by ID.
    pub branches: HashMap<FlowId, Branch>,

    /// The most recent reply text to display.
    pub last_reply: Option<String>,
}
