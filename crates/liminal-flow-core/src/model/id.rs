// Unique identifier type for Liminal Flow entities
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

/// A unique identifier backed by ULID, providing time-ordered, globally unique IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlowId(String);

impl FlowId {
    /// Generate a new unique identifier.
    pub fn new() -> Self {
        Self(Ulid::new().to_string().to_lowercase())
    }

    /// Create a FlowId from an existing string (e.g., loaded from the database).
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Return the inner string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for FlowId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FlowId {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for FlowId {
    fn from(s: &str) -> Self {
        Self::from_string(s.to_string())
    }
}
