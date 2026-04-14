#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;

use wallselect::cli::{Cli, Command};
use wallselect::config::{init_config, load_config, resolve_paths};
use wallselect::history;
use wallselect::scanner::scan_directories;
use wallselect::selector::select_random_thread_rng;
use wallselect::state::AppState;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = resolve_paths(cli.config.as_deref())?;

    match &cli.command {
        Command::InitConfig => {
            let created = init_config(&paths.config_file)?;
            if created {
                println!("{}", paths.config_file.display());
            } else {
                eprintln!("config already exists at {}", paths.config_file.display());
            }
            return Ok(());
        }
        _ => {}
    }

    let overrides = cli.command.config_overrides();
    let config = load_config(&paths.config_file, &overrides)?;

    if cli.verbose {
        eprintln!("using config {}", paths.config_file.display());
        eprintln!("using state {}", paths.state_file.display());
    }

    match cli.command {
        Command::Scan(_) => {
            let files = scan_directories(&config.dirs, config.recursive, &config.extensions)?;
            let mut state = AppState::load_or_default(&paths.state_file)?;
            state.set_scanned_files(files.clone());
            state.save(&paths.state_file)?;

            if cli.verbose {
                eprintln!("scanned {} candidate file(s)", files.len());
            }

            for file in files {
                println!("{}", file.display());
            }
        }
        Command::Random(_) => {
            let mut state = AppState::load_or_default(&paths.state_file)?;
            let should_rescan = overrides.affects_scan() || state.scanned_files.is_empty();
            let candidates = if should_rescan {
                let files = scan_directories(&config.dirs, config.recursive, &config.extensions)?;
                state.set_scanned_files(files.clone());
                files
            } else {
                state.scanned_files.clone()
            };

            let selected = select_random_thread_rng(
                &candidates,
                &state.history,
                config.random_no_repeat_window,
            )?;

            println!("{}", selected.display());

            state.record_selection(selected, config.history_size);
            state.save(&paths.state_file)?;
        }
        Command::History(args) => {
            let state = AppState::load_or_default(&paths.state_file)?;
            for entry in history::recent(&state.history, args.limit) {
                println!("{}", entry.display());
            }
        }
        Command::ShowConfig(_) => {
            print!("{}", config.to_toml_string()?);
        }
        Command::InitConfig => unreachable!("handled before config loading"),
    }

    Ok(())
}
