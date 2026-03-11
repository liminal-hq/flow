// Reply entity — the short status voice shown in the TUI
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A reply is the short status text shown in the reply pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub text: String,
    pub created_at: DateTime<Utc>,
}
