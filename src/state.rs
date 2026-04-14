//! Persisted application state.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::history;

/// Persisted scan results and selection history.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppState {
    /// The last known scan results.
    pub scanned_files: Vec<PathBuf>,
    /// Previously selected wallpapers in chronological order.
    pub history: Vec<PathBuf>,
    /// The current wallpaper selection.
    pub current: Option<PathBuf>,
}

impl AppState {
    /// Load state from disk, returning defaults if the file is missing or invalid.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        match toml::from_str(&contents) {
            Ok(state) => Ok(state),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Save the current state to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let serialized = toml::to_string_pretty(self).context("failed to serialize state")?;
        fs::write(path, serialized)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    /// Replace the cached scan result.
    pub fn set_scanned_files(&mut self, files: Vec<PathBuf>) {
        self.scanned_files = files;
    }

    /// Record the latest wallpaper selection and trim history to the configured size.
    pub fn record_selection(&mut self, selection: PathBuf, history_size: usize) {
        self.current = Some(selection.clone());
        history::push_selection(&mut self.history, selection, history_size);
    }
}
