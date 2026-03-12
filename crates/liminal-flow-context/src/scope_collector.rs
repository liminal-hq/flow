// Scope collection and attachment for threads and branches
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use liminal_flow_core::model::{FlowId, Scope, ScopeKind};

use crate::{cwd, git};

/// Collected context scopes for the current environment.
pub struct CollectedScopes {
    pub repo: Option<String>,
    pub git_branch: Option<String>,
    pub cwd: Option<String>,
}

/// Collect available scopes from the current environment.
pub fn collect() -> CollectedScopes {
    CollectedScopes {
        repo: git::repo_root(),
        git_branch: git::current_branch(),
        cwd: cwd::current_dir(),
    }
}

/// Convert collected scopes into Scope entities for a given target.
pub fn as_scopes(
    collected: &CollectedScopes,
    target_type: &str,
    target_id: &FlowId,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Vec<Scope> {
    let mut scopes = Vec::new();

    if let Some(ref repo) = collected.repo {
        scopes.push(Scope {
            id: FlowId::new(),
            target_type: target_type.to_string(),
            target_id: target_id.clone(),
            kind: ScopeKind::Repo,
            value: repo.clone(),
            confidence: 1.0,
            observed_at,
        });
    }

    if let Some(ref branch) = collected.git_branch {
        scopes.push(Scope {
            id: FlowId::new(),
            target_type: target_type.to_string(),
            target_id: target_id.clone(),
            kind: ScopeKind::GitBranch,
            value: branch.clone(),
            confidence: 1.0,
            observed_at,
        });
    }

    if let Some(ref dir) = collected.cwd {
        scopes.push(Scope {
            id: FlowId::new(),
            target_type: target_type.to_string(),
            target_id: target_id.clone(),
            kind: ScopeKind::Cwd,
            value: dir.clone(),
            confidence: 1.0,
            observed_at,
        });
    }

    scopes
}
