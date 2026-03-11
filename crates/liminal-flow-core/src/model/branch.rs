// Branch entity — a lightweight offshoot of attention within a thread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::FlowId;

/// The current lifecycle status of a branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    Active,
    Parked,
    Done,
    Dropped,
}

impl BranchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Parked => "parked",
            Self::Done => "done",
            Self::Dropped => "dropped",
        }
    }
}

impl std::fmt::Display for BranchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for BranchStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "parked" => Ok(Self::Parked),
            "done" => Ok(Self::Done),
            "dropped" => Ok(Self::Dropped),
            _ => Err(format!("unknown branch status: {s}")),
        }
    }
}

/// A branch is a temporary offshoot of attention beneath a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: FlowId,
    pub thread_id: FlowId,
    pub title: String,
    pub status: BranchStatus,
    pub short_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
