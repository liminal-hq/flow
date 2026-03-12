// Context error types for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("git discovery failed: {0}")]
    GitDiscovery(String),

    #[error("cwd not available: {0}")]
    CwdUnavailable(String),
}
