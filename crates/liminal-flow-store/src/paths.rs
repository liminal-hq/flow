// Platform-aware storage path resolution for Liminal Flow
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::StoreError;

/// Resolve the path for the SQLite database file.
///
/// Linux:   $XDG_DATA_HOME/liminal-flow/data.sqlite3
/// macOS:   ~/Library/Application Support/Liminal Flow/data.sqlite3
/// Windows: %LOCALAPPDATA%\Liminal Flow\data.sqlite3
pub fn data_dir() -> Result<PathBuf, StoreError> {
    let dirs = ProjectDirs::from("ca", "liminalhq", "liminal-flow")
        .ok_or_else(|| StoreError::PathResolution("could not determine data directory".into()))?;

    Ok(dirs.data_dir().to_path_buf())
}

/// Resolve the path for the config directory.
///
/// Linux:   $XDG_CONFIG_HOME/liminal-flow/
/// macOS:   ~/Library/Application Support/Liminal Flow/
/// Windows: %APPDATA%\Liminal Flow\
pub fn config_dir() -> Result<PathBuf, StoreError> {
    let dirs = ProjectDirs::from("ca", "liminalhq", "liminal-flow")
        .ok_or_else(|| StoreError::PathResolution("could not determine config directory".into()))?;

    Ok(dirs.config_dir().to_path_buf())
}

/// Return the full path to the SQLite database file.
pub fn database_path() -> Result<PathBuf, StoreError> {
    Ok(data_dir()?.join("data.sqlite3"))
}

/// Return the full path to the config file.
pub fn config_path() -> Result<PathBuf, StoreError> {
    Ok(config_dir()?.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_is_resolved() {
        let dir = data_dir().expect("should resolve data dir");
        let path_str = dir.to_string_lossy();
        assert!(
            path_str.contains("liminal-flow") || path_str.contains("Liminal Flow"),
            "data dir should contain the app identifier: {path_str}"
        );
    }

    #[test]
    fn database_path_ends_with_sqlite() {
        let path = database_path().expect("should resolve db path");
        assert!(
            path.to_string_lossy().ends_with("data.sqlite3"),
            "database path should end with data.sqlite3"
        );
    }
}
