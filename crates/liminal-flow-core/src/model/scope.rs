// Scope entity — structured context attached to a thread or branch
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::FlowId;

/// The kind of scope context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Repo,
    Cwd,
    GitBranch,
    Workspace,
    Host,
}

impl ScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Cwd => "cwd",
            Self::GitBranch => "git_branch",
            Self::Workspace => "workspace",
            Self::Host => "host",
        }
    }
}

impl std::str::FromStr for ScopeKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "repo" => Ok(Self::Repo),
            "cwd" => Ok(Self::Cwd),
            "git_branch" => Ok(Self::GitBranch),
            "workspace" => Ok(Self::Workspace),
            "host" => Ok(Self::Host),
            _ => Err(format!("unknown scope kind: {s}")),
        }
    }
}

/// A scope is structured context attached to a thread or branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: FlowId,
    pub target_type: String,
    pub target_id: FlowId,
    pub kind: ScopeKind,
    pub value: String,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
}
