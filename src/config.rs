//! Configuration defaults, loading, and path resolution.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

const CONFIG_FILE_NAME: &str = "config.toml";
const STATE_FILE_NAME: &str = "state.toml";

/// How a wallpaper image should be scaled to the output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApplyMode {
    /// Scale to fill the output and crop excess area.
    #[default]
    Fill,
}

/// Fully resolved application configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Wallpaper directories to scan.
    pub dirs: Vec<PathBuf>,
    /// Recurse into nested directories when scanning.
    pub recursive: bool,
    /// Allowed image file extensions.
    pub extensions: Vec<String>,
    /// Maximum number of history entries to retain.
    pub history_size: usize,
    /// Number of recent selections to avoid repeating when possible.
    pub random_no_repeat_window: usize,
    /// How wallpapers should be scaled when applied on Wayland.
    pub apply_mode: ApplyMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        let dirs = UserDirs::new()
            .and_then(|user_dirs| user_dirs.picture_dir().map(Path::to_path_buf))
            .into_iter()
            .collect();

        Self {
            dirs,
            recursive: true,
            extensions: vec![
                "jpg".into(),
                "jpeg".into(),
                "png".into(),
                "webp".into(),
                "bmp".into(),
                "gif".into(),
            ],
            history_size: 50,
            random_no_repeat_window: 5,
            apply_mode: ApplyMode::Fill,
        }
    }
}

/// Optional config values loaded from TOML before defaults are applied.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct RawConfig {
    dirs: Option<Vec<PathBuf>>,
    recursive: Option<bool>,
    extensions: Option<Vec<String>>,
    history_size: Option<usize>,
    random_no_repeat_window: Option<usize>,
    apply_mode: Option<ApplyMode>,
}

/// CLI-driven config overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigOverrides {
    /// Override wallpaper directories.
    pub dirs: Option<Vec<PathBuf>>,
    /// Override recursive scanning behavior.
    pub recursive: Option<bool>,
    /// Override allowed file extensions.
    pub extensions: Option<Vec<String>>,
    /// Override history size.
    pub history_size: Option<usize>,
    /// Override the no-repeat selection window.
    pub random_no_repeat_window: Option<usize>,
}

/// Resolved application file paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// The TOML config file path.
    pub config_file: PathBuf,
    /// The persisted state file path.
    pub state_file: PathBuf,
}

impl AppConfig {
    /// Serialize the config as pretty TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).context("failed to serialize config")
    }
}

impl RawConfig {
    fn apply_to(self, mut config: AppConfig) -> AppConfig {
        if let Some(dirs) = self.dirs {
            config.dirs = dirs;
        }
        if let Some(recursive) = self.recursive {
            config.recursive = recursive;
        }
        if let Some(extensions) = self.extensions {
            config.extensions = extensions;
        }
        if let Some(history_size) = self.history_size {
            config.history_size = history_size;
        }
        if let Some(random_no_repeat_window) = self.random_no_repeat_window {
            config.random_no_repeat_window = random_no_repeat_window;
        }
        if let Some(apply_mode) = self.apply_mode {
            config.apply_mode = apply_mode;
        }

        config
    }
}

impl ConfigOverrides {
    /// Apply CLI overrides to an existing config value.
    pub fn apply_to(&self, mut config: AppConfig) -> AppConfig {
        if let Some(dirs) = &self.dirs {
            config.dirs = dirs.clone();
        }
        if let Some(recursive) = self.recursive {
            config.recursive = recursive;
        }
        if let Some(extensions) = &self.extensions {
            config.extensions = extensions.clone();
        }
        if let Some(history_size) = self.history_size {
            config.history_size = history_size;
        }
        if let Some(random_no_repeat_window) = self.random_no_repeat_window {
            config.random_no_repeat_window = random_no_repeat_window;
        }

        config
    }

    /// Return `true` when overrides change how candidate files are scanned.
    pub fn affects_scan(&self) -> bool {
        self.dirs.is_some() || self.recursive.is_some() || self.extensions.is_some()
    }
}

/// Resolve the default config and state file locations.
pub fn resolve_paths(config_override: Option<&Path>) -> Result<AppPaths> {
    let project_dirs = ProjectDirs::from("dev", "beepaper", "beepaper")
        .ok_or(ConfigError::ProjectDirsUnavailable)?;

    let config_file = match config_override {
        Some(path) => path.to_path_buf(),
        None => project_dirs.config_dir().join(CONFIG_FILE_NAME),
    };

    let state_file = project_dirs.data_local_dir().join(STATE_FILE_NAME);

    Ok(AppPaths {
        config_file,
        state_file,
    })
}

/// Load config from disk, applying defaults and CLI overrides.
pub fn load_config(path: &Path, overrides: &ConfigOverrides) -> Result<AppConfig> {
    let raw = if path.exists() {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Some(toml::from_str::<RawConfig>(&contents).context("failed to parse config TOML")?)
    } else {
        None
    };

    Ok(merge_config(raw, overrides))
}

/// Create a default config file if one does not already exist.
pub fn init_config(path: &Path) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let contents = AppConfig::default().to_toml_string()?;
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

fn merge_config(raw: Option<RawConfig>, overrides: &ConfigOverrides) -> AppConfig {
    let default_config = AppConfig::default();
    let file_config = raw.map_or(default_config.clone(), |raw| raw.apply_to(default_config));
    overrides.apply_to(file_config)
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ApplyMode, ConfigOverrides, RawConfig, merge_config};
    use std::path::PathBuf;

    #[test]
    fn partial_config_uses_defaults_for_missing_values() {
        let raw: RawConfig = toml::from_str(
            r#"
            dirs = ["/tmp/walls"]
            recursive = false
            "#,
        )
        .expect("raw config should parse");

        let merged = merge_config(Some(raw), &ConfigOverrides::default());
        let defaults = AppConfig::default();

        assert_eq!(merged.dirs, vec![PathBuf::from("/tmp/walls")]);
        assert!(!merged.recursive);
        assert_eq!(merged.extensions, defaults.extensions);
        assert_eq!(merged.history_size, defaults.history_size);
        assert_eq!(
            merged.random_no_repeat_window,
            defaults.random_no_repeat_window
        );
        assert_eq!(merged.apply_mode, defaults.apply_mode);
    }

    #[test]
    fn cli_overrides_take_precedence() {
        let raw: RawConfig = toml::from_str(
            r#"
            history_size = 10
            random_no_repeat_window = 2
            "#,
        )
        .expect("raw config should parse");

        let overrides = ConfigOverrides {
            history_size: Some(3),
            random_no_repeat_window: Some(1),
            extensions: Some(vec!["png".into()]),
            ..ConfigOverrides::default()
        };

        let merged = merge_config(Some(raw), &overrides);

        assert_eq!(merged.history_size, 3);
        assert_eq!(merged.random_no_repeat_window, 1);
        assert_eq!(merged.extensions, vec!["png"]);
        assert_eq!(merged.apply_mode, ApplyMode::Fill);
    }

    #[test]
    fn apply_mode_parses_from_config() {
        let raw: RawConfig = toml::from_str(
            r#"
            apply_mode = "fill"
            "#,
        )
        .expect("raw config should parse");

        let merged = merge_config(Some(raw), &ConfigOverrides::default());

        assert_eq!(merged.apply_mode, ApplyMode::Fill);
    }
}
