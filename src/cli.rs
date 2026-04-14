//! Clap-derived CLI definitions.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::ConfigOverrides;

/// The `wallselect` command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "wallselect",
    version,
    about = "Discover and select wallpapers from configured directories"
)]
pub struct Cli {
    /// Use a custom config file path.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Enable verbose logging to stderr.
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported `wallselect` subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan configured wallpaper directories and cache the results.
    Scan(ScanArgs),
    /// Select a random wallpaper path from cached or freshly scanned files.
    Random(RandomArgs),
    /// Print recent wallpaper selections.
    History(HistoryArgs),
    /// Print the resolved configuration after defaults and overrides.
    ShowConfig(ShowConfigArgs),
    /// Create a default config file if one does not already exist.
    InitConfig,
}

/// Config-affecting CLI overrides shared by relevant commands.
#[derive(Debug, Clone, Default, Args)]
pub struct ConfigArgs {
    /// Override configured wallpaper directories.
    #[arg(long = "dir", value_name = "PATH")]
    pub dirs: Vec<PathBuf>,

    /// Force recursive scanning.
    #[arg(long, conflicts_with = "no_recursive")]
    pub recursive: bool,

    /// Disable recursive scanning.
    #[arg(long)]
    pub no_recursive: bool,

    /// Override allowed file extensions.
    #[arg(long = "extension", value_name = "EXT")]
    pub extensions: Vec<String>,

    /// Override the maximum number of history entries kept.
    #[arg(long)]
    pub history_size: Option<usize>,

    /// Override the no-repeat selection window.
    #[arg(long = "no-repeat-window")]
    pub no_repeat_window: Option<usize>,
}

/// Arguments for the `scan` command.
#[derive(Debug, Clone, Default, Args)]
pub struct ScanArgs {
    /// Command-specific config overrides.
    #[command(flatten)]
    pub config: ConfigArgs,
}

/// Arguments for the `random` command.
#[derive(Debug, Clone, Default, Args)]
pub struct RandomArgs {
    /// Command-specific config overrides.
    #[command(flatten)]
    pub config: ConfigArgs,
}

/// Arguments for the `show-config` command.
#[derive(Debug, Clone, Default, Args)]
pub struct ShowConfigArgs {
    /// Command-specific config overrides.
    #[command(flatten)]
    pub config: ConfigArgs,
}

/// Arguments for the `history` command.
#[derive(Debug, Clone, Default, Args)]
pub struct HistoryArgs {
    /// Limit the number of history entries printed.
    #[arg(long)]
    pub limit: Option<usize>,
}

impl ConfigArgs {
    /// Convert parsed CLI args into config overrides.
    pub fn to_overrides(&self) -> ConfigOverrides {
        let recursive = if self.recursive {
            Some(true)
        } else if self.no_recursive {
            Some(false)
        } else {
            None
        };

        ConfigOverrides {
            dirs: (!self.dirs.is_empty()).then(|| self.dirs.clone()),
            recursive,
            extensions: (!self.extensions.is_empty()).then(|| self.extensions.clone()),
            history_size: self.history_size,
            random_no_repeat_window: self.no_repeat_window,
        }
    }
}

impl Command {
    /// Return config overrides for commands that support them.
    pub fn config_overrides(&self) -> ConfigOverrides {
        match self {
            Command::Scan(args) => args.config.to_overrides(),
            Command::Random(args) => args.config.to_overrides(),
            Command::ShowConfig(args) => args.config.to_overrides(),
            Command::History(_) | Command::InitConfig => ConfigOverrides::default(),
        }
    }
}
