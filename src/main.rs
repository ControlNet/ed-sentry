use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use clap::Parser;
use ed_afk_dashboard::config::{AppConfig, CliConfigOverrides, RuntimeConfig};
use ed_afk_dashboard::event::{parse_journal_line, JournalEvent};
use ed_afk_dashboard::journal::{
    configured_recent_journal_file_choices, default_journal_folder, live_poll_interval,
    preload_journal_file, select_configured_journal_file, stream_journal_file, LiveTail,
};
use ed_afk_dashboard::monitor::EventMonitor;
use ed_afk_dashboard::terminal::{render_banner, set_platform_window_title, TerminalNotifier};
use ed_afk_dashboard::text::line_safe;

#[derive(Clone, Debug, Parser)]
#[command(name = "ed-afk-dashboard", version)]
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    match build_runtime_command(cli).and_then(run_command) {
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

    Ok(RuntimeCommand {
        mode,
        config: loaded.config.into_runtime(&overrides),
        warnings: loaded.warnings,
    })
}

fn run_command(command: RuntimeCommand) -> Result<(), AppError> {
    for warning in &command.warnings {
        eprintln!("Warning: {}", line_safe(warning));
    }

    if command.mode == Mode::Replay && command.config.reset_session {
        eprintln!("Warning: --reset-session has no effect in replay");
    }

    println!(
        "{}",
        render_banner("ED AFK Dashboard v260421 by CMDR PSIPAB")
    );
    println!();

    match command.mode {
        Mode::Watch => run_watch(&command.config)?,
        Mode::Replay => run_replay(&command.config)?,
    }

    Ok(())
}

fn run_watch(config: &RuntimeConfig) -> Result<(), AppError> {
    let set_file = select_watch_journal_file(config)?;
    let preload =
        preload_journal_file(&set_file, parse_journal_line).map_err(|error| AppError {
            message: error.to_string(),
        })?;
    print_startup(
        config,
        &set_file,
        startup_commander(&preload.records).as_deref(),
    );
    if config.debug {
        eprintln!(
            "Debug: preloaded {} lines from {} to byte offset {}",
            preload.records.len(),
            line_safe(&set_file.display().to_string()),
            preload.eof_offset
        );
    }
    let last_preload_timestamp = preload
        .records
        .iter()
        .filter_map(|record| record.result.as_ref().ok().map(JournalEvent::timestamp))
        .next_back();
    let mut tail = LiveTail::from_preload(&set_file, &preload);
    let mut monitor =
        EventMonitor::from_runtime_config(TerminalNotifier::stdout(&config.monitor), config);

    for record in preload.records {
        match record.result {
            Ok(event) => monitor.process_event(&event).map_err(|error| AppError {
                message: error.to_string(),
            })?,
            Err(error) => eprintln!(
                "Warning: Malformed journal line {} during preload: {}",
                record.line_number,
                line_safe(&error.message)
            ),
        }
    }
    if config.reset_session {
        monitor.state_mut().reset_session_counters();
        monitor
            .dispatcher_mut()
            .dispatch(ed_afk_dashboard::notifier::Notification::new(
                "session_reset",
                1,
                ed_afk_dashboard::notifier::AlertLevel::Info,
                Some("🔄".to_string()),
                "Session stats reset",
                "Session stats reset",
                chrono::Utc::now(),
            ))
            .map_err(|error| AppError {
                message: error.to_string(),
            })?;
    }
    if last_preload_timestamp.is_some() {
        monitor
            .start_monitor(&journal_filename(&set_file), chrono::Utc::now())
            .map_err(|error| AppError {
                message: error.to_string(),
            })?;
    }
    render_live_status_if_supported(&mut monitor, config)?;

    loop {
        thread::sleep(live_poll_interval(config));
        let poll = tail.poll(parse_journal_line).map_err(|error| AppError {
            message: error.to_string(),
        })?;
        for warning in poll.warnings {
            eprintln!("Warning: {}", line_safe(&warning.to_string()));
        }
        for record in poll.records {
            match record.result {
                Ok(event) => monitor.process_event(&event).map_err(|error| AppError {
                    message: error.to_string(),
                })?,
                Err(error) => eprintln!(
                    "Warning: Malformed journal line at byte offset {}: {}",
                    record.start_offset,
                    line_safe(&error.message)
                ),
            }
        }
        monitor
            .check_warnings_at(chrono::Utc::now(), false)
            .map_err(|error| AppError {
                message: error.to_string(),
            })?;
        render_live_status_if_supported(&mut monitor, config)?;
    }
}

fn select_watch_journal_file(config: &RuntimeConfig) -> Result<PathBuf, AppError> {
    if config.set_file.is_some() || !config.file_select {
        return select_configured_journal_file(config).map_err(|error| AppError {
            message: error.to_string(),
        });
    }

    let choices = configured_recent_journal_file_choices(config).map_err(|error| AppError {
        message: error.to_string(),
    })?;
    for choice in &choices {
        println!(
            "{}: {}",
            choice.number,
            line_safe(&choice.file.path.display().to_string())
        );
    }
    print!("Select Journal file [1-{}]: ", choices.len());
    io::stdout().flush().map_err(|error| AppError {
        message: error.to_string(),
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| AppError {
            message: error.to_string(),
        })?;
    let selected = input.trim().parse::<usize>().map_err(|_| AppError {
        message: format!("invalid Journal selection: {}", input.trim()),
    })?;

    choices
        .into_iter()
        .find(|choice| choice.number == selected)
        .map(|choice| choice.file.path)
        .ok_or_else(|| AppError {
            message: format!("Journal selection {selected} is out of range"),
        })
}

fn render_live_status_if_supported(
    monitor: &mut EventMonitor<TerminalNotifier<std::io::Stdout>>,
    config: &RuntimeConfig,
) -> Result<(), AppError> {
    if !config.monitor.live_status || !monitor.dispatcher().notifier().supports_status_line() {
        render_dynamic_title_if_enabled(monitor, config);
        return Ok(());
    }

    let status = monitor.render_status_line(chrono::Utc::now());
    render_dynamic_title_if_enabled(monitor, config);
    monitor
        .dispatcher_mut()
        .notifier_mut()
        .render_status(&status)
        .map_err(|error| AppError {
            message: error.to_string(),
        })
}

fn render_dynamic_title_if_enabled(
    monitor: &EventMonitor<TerminalNotifier<std::io::Stdout>>,
    config: &RuntimeConfig,
) {
    if config.monitor.dynamic_title {
        let title = monitor.render_dynamic_title(chrono::Utc::now());
        set_platform_window_title(&title);
    }
}

fn run_replay(config: &RuntimeConfig) -> Result<(), AppError> {
    let set_file = select_configured_journal_file(config).map_err(|error| AppError {
        message: error.to_string(),
    })?;
    if config.debug {
        eprintln!(
            "Debug: replaying Journal file {}",
            line_safe(&set_file.display().to_string())
        );
    }
    let mut replay_lines = 0_usize;
    let mut commander = None;
    let scan = stream_journal_file(&set_file, parse_journal_line, |record| {
        replay_lines += 1;
        if commander.is_none() {
            commander = match record.result.as_ref().ok() {
                Some(JournalEvent::Commander(event)) => event.name.clone(),
                Some(JournalEvent::LoadGame(event)) => event.commander.clone(),
                _ => None,
            };
        }
        Ok::<(), std::convert::Infallible>(())
    })
    .map_err(|error| AppError {
        message: error.to_string(),
    })?;
    print_startup(config, &set_file, commander.as_deref());
    if config.debug {
        eprintln!(
            "Debug: replay loaded {} lines to byte offset {}",
            replay_lines, scan.eof_offset
        );
    }
    let mut monitor =
        EventMonitor::from_runtime_config(TerminalNotifier::stdout(&config.monitor), config);
    let mut last_timestamp = None;

    stream_journal_file(&set_file, parse_journal_line, |record| {
        match record.result {
            Ok(event) => {
                last_timestamp = Some(event.timestamp());
                monitor
                    .process_event(&event)
                    .map_err(|error| error.to_string())?;
            }
            Err(error) => {
                eprintln!(
                    "Warning: Malformed journal line {}: {}",
                    record.line_number,
                    line_safe(&error.message)
                );
            }
        }
        Ok::<(), String>(())
    })
    .map_err(|error| AppError {
        message: error.to_string(),
    })?;

    if let Some(timestamp) = last_timestamp {
        monitor
            .start_monitor(&journal_filename(&set_file), timestamp)
            .map_err(|error| AppError {
                message: error.to_string(),
            })?;
        monitor
            .finish(&journal_filename(&set_file), timestamp)
            .map_err(|error| AppError {
                message: error.to_string(),
            })?;
    }
    Ok(())
}

fn print_startup(config: &RuntimeConfig, set_file: &std::path::Path, commander: Option<&str>) {
    println!(
        "Journal folder: {}",
        line_safe(&watch_journal_folder_display(config))
    );
    println!("Journal file: {}", line_safe(&journal_filename(set_file)));
    println!(
        "Commander name: {}",
        line_safe(commander.unwrap_or("[Unknown]"))
    );
    println!("Config profile: Default");
    println!("\nStarting... (Press Ctrl+C to stop)\n");
    println!("Info: Discord webhook missing or invalid - operating with terminal output only\n");
}

fn startup_commander(
    records: &[ed_afk_dashboard::journal::PreloadRecord<JournalEvent>],
) -> Option<String> {
    records
        .iter()
        .find_map(|record| match record.result.as_ref().ok()? {
            JournalEvent::Commander(event) => event.name.clone(),
            JournalEvent::LoadGame(event) => event.commander.clone(),
            _ => None,
        })
}

fn journal_filename(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

fn watch_journal_folder_display(config: &RuntimeConfig) -> String {
    if !config.journal.folder.is_empty() {
        return config.journal.folder.clone();
    }

    default_journal_folder()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<Windows Saved Games known folder unavailable>".to_string())
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
        assert!(help.contains("--poll-interval-ms <ms>"));
        assert!(help.contains("--replay"));
        assert!(!help.contains("Commands:"));
    }

    #[test]
    fn cli_config_watch_accepts_poll_interval() {
        let cli = Cli::try_parse_from([
            "ed-afk-dashboard",
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
    fn cli_config_replay_flag_enables_replay_mode() {
        let cli = Cli::try_parse_from([
            "ed-afk-dashboard",
            "--replay",
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
    fn cli_config_without_replay_defaults_to_watch() {
        let cli = Cli::try_parse_from([
            "ed-afk-dashboard",
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
    fn cli_config_watch_display_uses_explicit_folder() {
        let config = RuntimeConfig {
            journal: ed_afk_dashboard::config::JournalConfig {
                folder: "/journals".to_string(),
                recent_files: 10,
            },
            monitor: Default::default(),
            log_levels: Default::default(),
            set_file: None,
            file_select: false,
            reset_session: false,
            debug: false,
        };

        assert_eq!(watch_journal_folder_display(&config), "/journals");
    }

    #[cfg(not(windows))]
    #[test]
    fn cli_config_watch_display_explains_unavailable_default_folder() {
        let config = RuntimeConfig {
            journal: Default::default(),
            monitor: Default::default(),
            log_levels: Default::default(),
            set_file: None,
            file_select: false,
            reset_session: false,
            debug: false,
        };

        assert_eq!(
            watch_journal_folder_display(&config),
            "<Windows Saved Games known folder unavailable>"
        );
    }

    #[test]
    fn cli_config_replay_rejects_poll_interval_ms() {
        let cli = Cli::try_parse_from([
            "ed-afk-dashboard",
            "--replay",
            "--poll-interval-ms",
            "1000",
            "--set-file",
            "Journal.log",
        ])
        .unwrap();
        let error = build_runtime_command(cli).unwrap_err();

        assert!(error.message.contains("--poll-interval-ms has no effect"));
    }

    #[test]
    fn cli_config_replay_requires_set_file() {
        let cli = Cli::try_parse_from(["ed-afk-dashboard", "--replay"]).unwrap();
        let error = build_runtime_command(cli).unwrap_err();

        assert!(error.message.contains("replay requires --set-file"));
    }

    #[test]
    fn cli_config_replay_rejects_journal_folder() {
        let cli = Cli::try_parse_from([
            "ed-afk-dashboard",
            "--replay",
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
