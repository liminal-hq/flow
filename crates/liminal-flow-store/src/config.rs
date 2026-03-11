// Configuration loading and saving for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Application configuration loaded from config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub context: ContextConfig,

    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            context: ContextConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_scopes: bool,

    #[serde(default)]
    pub show_hints: bool,

    #[serde(default)]
    pub compact_mode: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_scopes: true,
            show_hints: false,
            compact_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default)]
    pub shell_helper_enabled: bool,

    #[serde(default = "default_true")]
    pub git_enrichment: bool,

    #[serde(default)]
    pub ambient_hints: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            shell_helper_enabled: false,
            git_enrichment: true,
            ambient_hints: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Load config from the default path, falling back to defaults if the file doesn't exist.
pub fn load_config() -> FlowConfig {
    let path = match crate::paths::config_path() {
        Ok(p) => p,
        Err(_) => return FlowConfig::default(),
    };

    if !path.exists() {
        return FlowConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("invalid config file, using defaults: {e}");
            FlowConfig::default()
        }),
        Err(_) => FlowConfig::default(),
    }
}
