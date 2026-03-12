// Core error types for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("no active thread")]
    NoActiveThread,

    #[error("no active branch")]
    NoActiveBranch,

    #[error("thread not found: {0}")]
    ThreadNotFound(String),

    #[error("branch not found: {0}")]
    BranchNotFound(String),
}
