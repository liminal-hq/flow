// Hint entity — low-confidence observed context
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::FlowId;

/// The kind of ambient hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintKind {
    Process,
    Command,
    Tty,
    Activity,
}

impl HintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::Command => "command",
            Self::Tty => "tty",
            Self::Activity => "activity",
        }
    }
}

impl std::str::FromStr for HintKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "process" => Ok(Self::Process),
            "command" => Ok(Self::Command),
            "tty" => Ok(Self::Tty),
            "activity" => Ok(Self::Activity),
            _ => Err(format!("unknown hint kind: {s}")),
        }
    }
}

/// A hint is a lower-confidence observed piece of context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub id: FlowId,
    pub kind: HintKind,
    pub value: String,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
}
