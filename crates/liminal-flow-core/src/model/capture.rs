// Capture entity — raw user input before or alongside refinement
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::FlowId;

/// Where the capture originated from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    Keyboard,
    Cli,
    Voice,
    Import,
    System,
}

impl CaptureSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Cli => "cli",
            Self::Voice => "voice",
            Self::Import => "import",
            Self::System => "system",
        }
    }
}

impl std::str::FromStr for CaptureSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "keyboard" => Ok(Self::Keyboard),
            "cli" => Ok(Self::Cli),
            "voice" => Ok(Self::Voice),
            "import" => Ok(Self::Import),
            "system" => Ok(Self::System),
            _ => Err(format!("unknown capture source: {s}")),
        }
    }
}

/// The inferred intent of a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    SetCurrentThread,
    StartBranch,
    ReturnToParent,
    AddNote,
    QueryCurrent,
    Resume,
    Pause,
    Park,
    Done,
    Archive,
    Ambiguous,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SetCurrentThread => "set_current_thread",
            Self::StartBranch => "start_branch",
            Self::ReturnToParent => "return_to_parent",
            Self::AddNote => "add_note",
            Self::QueryCurrent => "query_current",
            Self::Resume => "resume",
            Self::Pause => "pause",
            Self::Park => "park",
            Self::Done => "done",
            Self::Archive => "archive",
            Self::Ambiguous => "ambiguous",
        }
    }
}

impl std::str::FromStr for Intent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set_current_thread" => Ok(Self::SetCurrentThread),
            "start_branch" => Ok(Self::StartBranch),
            "return_to_parent" => Ok(Self::ReturnToParent),
            "add_note" => Ok(Self::AddNote),
            "query_current" => Ok(Self::QueryCurrent),
            "resume" => Ok(Self::Resume),
            "pause" => Ok(Self::Pause),
            "park" => Ok(Self::Park),
            "done" => Ok(Self::Done),
            "archive" => Ok(Self::Archive),
            "ambiguous" => Ok(Self::Ambiguous),
            _ => Err(format!("unknown intent: {s}")),
        }
    }
}

/// A capture represents raw user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capture {
    pub id: FlowId,
    pub target_type: String,
    pub target_id: FlowId,
    pub text: String,
    pub source: CaptureSource,
    pub inferred_intent: Option<Intent>,
    pub created_at: DateTime<Utc>,
}
