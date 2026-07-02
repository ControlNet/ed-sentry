use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::app::runtime::{run_replay, run_watch, RuntimeError};
use crate::build_info::app_title;
use crate::config::{AppConfig, CliConfigOverrides, RuntimeConfig};
use crate::terminal::render_banner;
use crate::text::line_safe;

#[derive(Clone, Debug, Parser)]
#[command(name = "ed-sentry", version)]
pub struct Cli {
    #[arg(long, value_name = "folder", global = true)]
    journal: Option<PathBuf>,

    #[arg(long, value_name = "file", global = true)]
    set_file: Option<PathBuf>,

    #[arg(long, global = true)]
    file_select: bool,

    #[arg(long, global = true)]
    reset_session: bool,

    #[arg(long, global = true)]
    debug: bool,

    #[arg(long, value_name = "file", global = true)]
    config: Option<PathBuf>,

    #[arg(long, global = true)]
    no_status_line: bool,

    #[arg(long, value_name = "ms")]
    poll_interval_ms: Option<u64>,

    #[arg(long)]
    replay: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    Watch,
    Replay,
}

#[derive(Debug)]
struct RuntimeCommand {
    mode: Mode,
    config: RuntimeConfig,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct AppError {
    message: String,
}

impl From<RuntimeError> for AppError {
    fn from(error: RuntimeError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

pub async fn run_from_env() -> ExitCode {
    let cli = Cli::parse();
    let result = match build_runtime_command(cli) {
        Ok(command) => run_command(command).await,
        Err(error) => Err(error),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {}", line_safe(&error.message));
            ExitCode::from(1)
        }
    }
}

fn build_runtime_command(cli: Cli) -> Result<RuntimeCommand, AppError> {
    let mode = if cli.replay {
        Mode::Replay
    } else {
        Mode::Watch
    };

    if mode == Mode::Replay && cli.journal.is_some() {
        return Err(AppError {
            message: "replay does not accept --journal; use --set-file <file>".to_string(),
        });
    }
    if mode == Mode::Replay && cli.poll_interval_ms.is_some() {
        return Err(AppError {
            message: "--poll-interval-ms has no effect with --replay".to_string(),
        });
    }
    if mode == Mode::Replay && cli.set_file.is_none() {
        return Err(AppError {
            message: "replay requires --set-file <file>".to_string(),
        });
    }

    let loaded = AppConfig::load_optional(cli.config.as_deref()).map_err(|error| AppError {
        message: error.to_string(),
    })?;

    let overrides = CliConfigOverrides {
        journal_folder: cli.journal.clone(),
        set_file: cli.set_file.clone(),
        file_select: cli.file_select,
        reset_session: cli.reset_session,
        debug: cli.debug,
        no_status_line: cli.no_status_line,
        poll_interval_ms: cli.poll_interval_ms,
    };

    let config_source = loaded.source;

    Ok(RuntimeCommand {
        mode,
        config: loaded
            .config
            .into_runtime_with_source(config_source, &overrides),
        warnings: loaded.warnings,
    })
}

async fn run_command(command: RuntimeCommand) -> Result<(), AppError> {
    for warning in &command.warnings {
        eprintln!("Warning: {}", line_safe(warning));
    }

    if command.mode == Mode::Replay && command.config.reset_session {
        eprintln!("Warning: --reset-session has no effect in replay");
    }

    println!(
        "{}",
        render_banner(&format!("{} by CMDR ControlNet", app_title()))
    );
    println!();

    match command.mode {
        Mode::Watch => run_watch(&command.config).await?,
        Mode::Replay => run_replay(&command.config).await?,
    }

    Ok(())
}

#[cfg(test)]
mod tests;
