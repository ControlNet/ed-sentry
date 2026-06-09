use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ed_afk_watch::config::{AppConfig, CliConfigOverrides, RuntimeConfig};

#[derive(Clone, Debug, Parser)]
#[command(name = "ed-afk-watch", version)]
struct Cli {
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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
    Watch(WatchArgs),
    Replay,
}

#[derive(Clone, Debug, Parser)]
struct WatchArgs {
    #[arg(long, value_name = "ms")]
    poll_interval_ms: Option<u64>,
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    match build_runtime_command(cli).and_then(run_command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {}", error.message);
            ExitCode::from(1)
        }
    }
}

fn build_runtime_command(cli: Cli) -> Result<RuntimeCommand, AppError> {
    let mode = match &cli.command {
        Some(Commands::Replay) => Mode::Replay,
        Some(Commands::Watch(_)) | None => Mode::Watch,
    };

    if mode == Mode::Replay && cli.journal.is_some() {
        return Err(AppError {
            message: "replay does not accept --journal; use --set-file <file>".to_string(),
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

    let poll_interval_ms = match &cli.command {
        Some(Commands::Watch(watch)) => watch.poll_interval_ms,
        _ => None,
    };
    let overrides = CliConfigOverrides {
        journal_folder: cli.journal.clone(),
        set_file: cli.set_file.clone(),
        file_select: cli.file_select,
        reset_session: cli.reset_session,
        debug: cli.debug,
        no_status_line: cli.no_status_line,
        poll_interval_ms,
    };

    Ok(RuntimeCommand {
        mode,
        config: loaded.config.into_runtime(&overrides),
        warnings: loaded.warnings,
    })
}

fn run_command(command: RuntimeCommand) -> Result<(), AppError> {
    for warning in &command.warnings {
        eprintln!("warning: {warning}");
    }

    if command.mode == Mode::Replay && command.config.reset_session {
        eprintln!("warning: --reset-session has no effect in replay");
    }

    match command.mode {
        Mode::Watch => {
            println!(
                "watch configuration loaded: journal='{}' poll_interval_ms={} live_status={}",
                command.config.journal.folder,
                command.config.monitor.poll_interval_ms,
                command.config.monitor.live_status
            );
        }
        Mode::Replay => {
            let set_file = command
                .config
                .set_file
                .as_ref()
                .expect("replay set_file is validated before runtime dispatch");
            println!(
                "replay configuration loaded: set_file='{}' live_status={}",
                set_file.display(),
                command.config.monitor.live_status
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_config_help_lists_locked_flags() {
        let help = Cli::command().render_help().to_string();

        assert!(help.contains("--journal <folder>"));
        assert!(help.contains("--set-file <file>"));
        assert!(help.contains("--file-select"));
        assert!(help.contains("--reset-session"));
        assert!(help.contains("--debug"));
        assert!(help.contains("--config <file>"));
        assert!(help.contains("--no-status-line"));
        assert!(help.contains("watch"));
        assert!(help.contains("replay"));
    }

    #[test]
    fn cli_config_watch_accepts_poll_interval() {
        let cli = Cli::try_parse_from([
            "ed-afk-watch",
            "watch",
            "--journal",
            "/journals",
            "--poll-interval-ms",
            "250",
        ])
        .unwrap();
        let command = build_runtime_command(cli).unwrap();

        assert_eq!(command.mode, Mode::Watch);
        assert_eq!(command.config.journal.folder, "/journals");
        assert_eq!(command.config.monitor.poll_interval_ms, 250);
    }

    #[test]
    fn cli_config_global_flags_work_before_replay_subcommand() {
        let cli = Cli::try_parse_from([
            "ed-afk-watch",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "replay",
            "--no-status-line",
        ])
        .unwrap();
        let command = build_runtime_command(cli).unwrap();

        assert_eq!(command.mode, Mode::Replay);
        assert_eq!(
            command.config.set_file,
            Some(PathBuf::from("tests/fixtures/journal_combat_bounty.log"))
        );
        assert!(!command.config.monitor.live_status);
    }

    #[test]
    fn cli_config_global_flags_work_after_replay_subcommand() {
        let cli = Cli::try_parse_from([
            "ed-afk-watch",
            "replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .unwrap();
        let command = build_runtime_command(cli).unwrap();

        assert_eq!(command.mode, Mode::Replay);
        assert_eq!(
            command.config.set_file,
            Some(PathBuf::from("tests/fixtures/journal_combat_bounty.log"))
        );
        assert!(!command.config.monitor.live_status);
    }

    #[test]
    fn cli_config_no_subcommand_watch_alias() {
        let cli = Cli::try_parse_from([
            "ed-afk-watch",
            "--journal",
            "/journals",
            "--set-file",
            "Journal.log",
        ])
        .unwrap();
        let command = build_runtime_command(cli).unwrap();

        assert_eq!(command.mode, Mode::Watch);
        assert_eq!(command.config.journal.folder, "/journals");
        assert_eq!(command.config.set_file, Some(PathBuf::from("Journal.log")));
    }

    #[test]
    fn cli_config_replay_rejects_poll_interval_ms() {
        let error = Cli::try_parse_from([
            "ed-afk-watch",
            "replay",
            "--poll-interval-ms",
            "1000",
            "--set-file",
            "Journal.log",
        ])
        .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn cli_config_replay_requires_set_file() {
        let cli = Cli::try_parse_from(["ed-afk-watch", "replay"]).unwrap();
        let error = build_runtime_command(cli).unwrap_err();

        assert!(error.message.contains("replay requires --set-file"));
    }

    #[test]
    fn cli_config_replay_rejects_journal_folder() {
        let cli = Cli::try_parse_from([
            "ed-afk-watch",
            "replay",
            "--journal",
            "/journals",
            "--set-file",
            "Journal.log",
        ])
        .unwrap();
        let error = build_runtime_command(cli).unwrap_err();

        assert!(error.message.contains("replay does not accept --journal"));
    }
}
