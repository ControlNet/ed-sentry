use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use ed_sentry::config::{AppConfig, CliConfigOverrides, MatrixRuntimeConfig, RuntimeConfig};
use ed_sentry::delivery::{DeliveryHub, DeliveryWarning, RemoteDelivery, StatusCadence};
use ed_sentry::event::{parse_journal_line, JournalEvent};
use ed_sentry::journal::{
    configured_recent_journal_file_choices, default_journal_folder, live_poll_interval,
    preload_journal_file, select_configured_journal_file, stream_journal_file, LiveTail,
};
use ed_sentry::matrix::MatrixDelivery;
use ed_sentry::monitor::EventMonitor;
use ed_sentry::notifier::{AlertLevel, Notification};
use ed_sentry::terminal::{render_banner, set_platform_window_title, TerminalNotifier};
use ed_sentry::text::line_safe;

#[derive(Clone, Debug, Parser)]
#[command(name = "ed-sentry", version)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatrixStartupStatus {
    Disabled,
    Enabled,
    Unavailable,
}

struct WatchDelivery {
    hub: DeliveryHub<std::io::Stdout>,
    matrix_status: MatrixStartupStatus,
}

#[tokio::main]
async fn main() -> ExitCode {
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

    Ok(RuntimeCommand {
        mode,
        config: loaded.config.into_runtime(&overrides),
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

    println!("{}", render_banner("ed-sentry v260421 by CMDR ControlNet"));
    println!();

    match command.mode {
        Mode::Watch => run_watch(&command.config).await?,
        Mode::Replay => run_replay(&command.config).await?,
    }

    Ok(())
}

async fn run_watch(config: &RuntimeConfig) -> Result<(), AppError> {
    let program_started_at = chrono::Utc::now();
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
    let WatchDelivery {
        hub: mut delivery,
        matrix_status,
    } = build_watch_delivery(config).await;
    print_matrix_startup_status(matrix_status);
    send_matrix_startup_header(&mut delivery, config, &set_file, program_started_at).await?;
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
    let mut monitor = EventMonitor::from_runtime_config(config);

    for record in preload.records {
        match record.result {
            Ok(event) => {
                let event_timestamp = event.timestamp();
                let notifications = monitor.process_event(&event);
                if event_timestamp < program_started_at {
                    deliver_terminal_notifications(&mut delivery, &notifications)?;
                } else {
                    deliver_notifications(&mut delivery, &notifications).await?;
                }
            }
            Err(error) => eprintln!(
                "Warning: Malformed journal line {} during preload: {}",
                record.line_number,
                line_safe(&error.message)
            ),
        }
    }
    if config.reset_session {
        monitor.state_mut().reset_session_counters();
        let notifications = [Notification::new(
            "session_reset",
            1,
            AlertLevel::Info,
            Some("🔄".to_string()),
            "Session stats reset",
            "Session stats reset",
            chrono::Utc::now(),
        )];
        deliver_notifications(&mut delivery, &notifications).await?;
    }
    if last_preload_timestamp.is_some() {
        let notifications =
            [monitor.start_monitor(&journal_filename(&set_file), chrono::Utc::now())];
        deliver_notifications(&mut delivery, &notifications).await?;
    }
    render_live_status_if_supported(&monitor, &mut delivery, config, true).await?;

    loop {
        tokio::time::sleep(live_poll_interval(config)).await;
        let poll = tail.poll(parse_journal_line).map_err(|error| AppError {
            message: error.to_string(),
        })?;
        for warning in poll.warnings {
            eprintln!("Warning: {}", line_safe(&warning.to_string()));
        }
        for record in poll.records {
            match record.result {
                Ok(event) => {
                    let notifications = monitor.process_event(&event);
                    deliver_notifications(&mut delivery, &notifications).await?;
                }
                Err(error) => eprintln!(
                    "Warning: Malformed journal line at byte offset {}: {}",
                    record.start_offset,
                    line_safe(&error.message)
                ),
            }
        }
        let notifications = monitor.check_warnings_at(chrono::Utc::now(), false);
        deliver_notifications(&mut delivery, &notifications).await?;
        render_live_status_if_supported(&monitor, &mut delivery, config, false).await?;
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

async fn render_live_status_if_supported(
    monitor: &EventMonitor,
    delivery: &mut DeliveryHub<std::io::Stdout>,
    config: &RuntimeConfig,
    force_status_publish: bool,
) -> Result<(), AppError> {
    if !config.monitor.live_status {
        render_dynamic_title_if_enabled(monitor, config);
        return Ok(());
    }

    let now = chrono::Utc::now();
    let status = monitor.render_status_line(now);
    render_dynamic_title_if_enabled(monitor, config);
    let render_terminal_status = delivery.supports_status_line();
    let warnings = delivery
        .publish_status(&status, now, force_status_publish, render_terminal_status)
        .await
        .map_err(|error| AppError {
            message: error.to_string(),
        })?;
    print_delivery_warnings(warnings);
    Ok(())
}

async fn send_matrix_startup_header(
    delivery: &mut DeliveryHub<std::io::Stdout>,
    config: &RuntimeConfig,
    set_file: &std::path::Path,
    program_started_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
    let notifications = [matrix_startup_header_notification(
        config,
        set_file,
        program_started_at,
    )];
    let warnings = delivery
        .send_remote_notifications(&notifications)
        .await
        .map_err(|error| AppError {
            message: error.to_string(),
        })?;
    print_delivery_warnings(warnings);
    Ok(())
}

fn matrix_startup_header_notification(
    config: &RuntimeConfig,
    set_file: &std::path::Path,
    program_started_at: chrono::DateTime<chrono::Utc>,
) -> Notification {
    let matrix_room = config
        .matrix
        .as_ref()
        .and_then(|matrix| matrix.room_id.as_deref())
        .unwrap_or("[disabled]");
    let remote_text = format!(
        "🛰️ ed-sentry started\nVersion: {}\nStarted at: {}\nJournal folder: {}\nJournal file: {}\nMatrix room: {}",
        env!("CARGO_PKG_VERSION"),
        program_started_at.to_rfc3339(),
        watch_journal_folder_display(config),
        journal_filename(set_file),
        matrix_room,
    );

    Notification::new(
        "matrix_startup",
        1,
        AlertLevel::Info,
        None,
        remote_text.clone(),
        remote_text,
        program_started_at,
    )
}

async fn build_watch_delivery(config: &RuntimeConfig) -> WatchDelivery {
    let terminal = TerminalNotifier::stdout(&config.monitor);
    let (matrix_status, matrix) = connect_matrix_delivery(config).await;
    WatchDelivery {
        hub: DeliveryHub::new(terminal, matrix)
            .with_status_cadence(status_cadence_from_config(config)),
        matrix_status,
    }
}

async fn connect_matrix_delivery(
    config: &RuntimeConfig,
) -> (MatrixStartupStatus, Option<Box<dyn RemoteDelivery>>) {
    let runtime = config.matrix_runtime();
    for warning in runtime.warnings {
        eprintln!(
            "Warning: Matrix delivery disabled: {}",
            line_safe(&matrix_validation_reason(&warning))
        );
    }

    let Some(matrix_config) = runtime.config else {
        let status = if config.matrix.is_some() {
            MatrixStartupStatus::Unavailable
        } else {
            MatrixStartupStatus::Disabled
        };
        return (status, None);
    };

    match connect_matrix_delivery_runtime(matrix_config.clone()).await {
        Ok(matrix) => (MatrixStartupStatus::Enabled, Some(matrix)),
        Err(error) => {
            eprintln!(
                "Warning: Matrix delivery disabled: {}",
                redact_matrix_error_message(&error, &matrix_config.access_token)
            );
            (MatrixStartupStatus::Unavailable, None)
        }
    }
}

async fn connect_matrix_delivery_runtime(
    config: MatrixRuntimeConfig,
) -> anyhow::Result<Box<dyn RemoteDelivery>> {
    #[cfg(debug_assertions)]
    if let Some(delivery) = debug_matrix_delivery_from_env(&config).await? {
        return Ok(delivery);
    }

    Ok(Box::new(MatrixDelivery::connect(config).await?))
}

fn matrix_validation_reason(warning: &str) -> String {
    warning
        .strip_prefix("Matrix delivery disabled for this run: ")
        .unwrap_or(warning)
        .to_string()
}

fn redact_matrix_error_message(error: &anyhow::Error, access_token: &str) -> String {
    let message = line_safe(&error.to_string());
    if access_token.is_empty() {
        message
    } else {
        message.replace(access_token, "<redacted>")
    }
}

fn render_dynamic_title_if_enabled(monitor: &EventMonitor, config: &RuntimeConfig) {
    if config.monitor.dynamic_title {
        let title = monitor.render_dynamic_title(chrono::Utc::now());
        set_platform_window_title(&title);
    }
}

fn status_cadence_from_config(config: &RuntimeConfig) -> StatusCadence {
    StatusCadence::new(Duration::from_secs(
        config
            .matrix
            .as_ref()
            .map(|matrix| matrix.status_update_interval_seconds)
            .unwrap_or(60),
    ))
}

async fn run_replay(config: &RuntimeConfig) -> Result<(), AppError> {
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
    let mut delivery = DeliveryHub::terminal_only(TerminalNotifier::stdout(&config.monitor));
    let mut monitor = EventMonitor::from_runtime_config(config);
    let mut last_timestamp = None;
    let mut replay_notifications = Vec::new();

    stream_journal_file(&set_file, parse_journal_line, |record| {
        match record.result {
            Ok(event) => {
                last_timestamp = Some(event.timestamp());
                replay_notifications.extend(monitor.process_event(&event));
            }
            Err(error) => {
                eprintln!(
                    "Warning: Malformed journal line {}: {}",
                    record.line_number,
                    line_safe(&error.message)
                );
            }
        }
        Ok::<(), std::convert::Infallible>(())
    })
    .map_err(|error| AppError {
        message: error.to_string(),
    })?;

    if let Some(timestamp) = last_timestamp {
        replay_notifications.push(monitor.start_monitor(&journal_filename(&set_file), timestamp));
        replay_notifications.extend(monitor.finish(&journal_filename(&set_file), timestamp));
    }
    deliver_notifications(&mut delivery, &replay_notifications).await?;
    Ok(())
}

async fn deliver_notifications(
    delivery: &mut DeliveryHub<std::io::Stdout>,
    notifications: &[Notification],
) -> Result<(), AppError> {
    let warnings = delivery
        .send_notifications(notifications)
        .await
        .map_err(|error| AppError {
            message: error.to_string(),
        })?;
    print_delivery_warnings(warnings);
    Ok(())
}

fn deliver_terminal_notifications(
    delivery: &mut DeliveryHub<std::io::Stdout>,
    notifications: &[Notification],
) -> Result<(), AppError> {
    delivery
        .send_terminal_notifications(notifications)
        .map_err(|error| AppError {
            message: error.to_string(),
        })
}

fn print_delivery_warnings(warnings: Vec<DeliveryWarning>) {
    for warning in warnings {
        eprintln!("Warning: {}", line_safe(&warning.message));
    }
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
}

fn print_matrix_startup_status(status: MatrixStartupStatus) {
    match status {
        MatrixStartupStatus::Disabled => {
            println!("Info: Matrix delivery disabled - operating with terminal output only\n");
        }
        MatrixStartupStatus::Enabled => {
            println!("Info: Matrix delivery enabled\n");
        }
        MatrixStartupStatus::Unavailable => {
            println!("Info: Matrix delivery unavailable - operating with terminal output only\n");
        }
    }
}

#[cfg(debug_assertions)]
async fn debug_matrix_delivery_from_env(
    config: &MatrixRuntimeConfig,
) -> anyhow::Result<Option<Box<dyn RemoteDelivery>>> {
    let Some(log_path) = std::env::var_os("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG") else {
        return Ok(None);
    };

    if let Ok(message) = std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_CONNECT_ERROR") {
        anyhow::bail!(message);
    }

    let delivery = DebugFileMatrixDelivery::new(std::path::PathBuf::from(log_path), config.clone());
    delivery.append_record(serde_json::json!({
        "kind": "connect",
        "homeserver": &config.homeserver,
        "room_id": &config.room_id,
        "mention_user_id": &config.mention_user_id,
    }))?;
    Ok(Some(Box::new(delivery)))
}

#[cfg(debug_assertions)]
struct DebugFileMatrixDelivery {
    log_path: std::path::PathBuf,
    access_token: String,
    send_error: Option<String>,
    status_error: Option<String>,
    send_delay: Duration,
    status_delay: Duration,
}

#[cfg(debug_assertions)]
impl DebugFileMatrixDelivery {
    fn new(log_path: std::path::PathBuf, config: MatrixRuntimeConfig) -> Self {
        Self {
            log_path,
            access_token: config.access_token,
            send_error: std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_ERROR").ok(),
            status_error: std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_STATUS_ERROR").ok(),
            send_delay: debug_delay_from_env("ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_DELAY_MS"),
            status_delay: debug_delay_from_env("ED_AFK_DASHBOARD_FAKE_MATRIX_STATUS_DELAY_MS"),
        }
    }

    fn append_record(&self, record: serde_json::Value) -> anyhow::Result<()> {
        use std::io::Write as _;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{}", serde_json::to_string(&record)?)?;
        Ok(())
    }

    fn failure(&self, message: &str) -> anyhow::Error {
        anyhow::anyhow!(line_safe(message).replace(&self.access_token, "<redacted>"))
    }
}

#[cfg(debug_assertions)]
#[async_trait::async_trait]
impl RemoteDelivery for DebugFileMatrixDelivery {
    async fn send(&mut self, notification: &Notification) -> anyhow::Result<()> {
        if !self.send_delay.is_zero() {
            tokio::time::sleep(self.send_delay).await;
        }
        if let Some(message) = &self.send_error {
            return Err(self.failure(message));
        }

        self.append_record(serde_json::json!({
            "kind": "send",
            "event_type": notification.event_type,
            "level": notification.level,
            "mention": notification.mention,
            "remote_text": notification.remote_text,
        }))
    }

    async fn publish_status(
        &mut self,
        status: &str,
        _now: chrono::DateTime<chrono::Utc>,
        force: bool,
    ) -> anyhow::Result<()> {
        if !self.status_delay.is_zero() {
            tokio::time::sleep(self.status_delay).await;
        }
        if let Some(message) = &self.status_error {
            return Err(self.failure(message));
        }

        self.append_record(serde_json::json!({
            "kind": "status",
            "status": line_safe(status),
            "force": force,
        }))
    }
}

#[cfg(debug_assertions)]
fn debug_delay_from_env(name: &str) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::ZERO)
}

fn startup_commander(
    records: &[ed_sentry::journal::PreloadRecord<JournalEvent>],
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
            "ed-sentry",
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
            "ed-sentry",
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
            "ed-sentry",
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
            journal: ed_sentry::config::JournalConfig {
                folder: "/journals".to_string(),
                recent_files: 10,
            },
            monitor: Default::default(),
            log_levels: Default::default(),
            matrix: None,
            set_file: None,
            file_select: false,
            reset_session: false,
            debug: false,
        };

        assert_eq!(watch_journal_folder_display(&config), "/journals");
    }

    #[test]
    fn cli_config_status_cadence_uses_matrix_interval_or_default() {
        let default_config = RuntimeConfig {
            journal: Default::default(),
            monitor: Default::default(),
            log_levels: Default::default(),
            matrix: None,
            set_file: None,
            file_select: false,
            reset_session: false,
            debug: false,
        };
        let matrix_config = RuntimeConfig {
            matrix: Some(ed_sentry::config::MatrixConfig {
                status_update_interval_seconds: 45,
                ..ed_sentry::config::MatrixConfig::default()
            }),
            ..default_config.clone()
        };

        assert_eq!(
            status_cadence_from_config(&default_config),
            StatusCadence::from_interval_seconds(60)
        );
        assert_eq!(
            status_cadence_from_config(&matrix_config),
            StatusCadence::from_interval_seconds(45)
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn cli_config_watch_display_explains_unavailable_default_folder() {
        let config = RuntimeConfig {
            journal: Default::default(),
            monitor: Default::default(),
            log_levels: Default::default(),
            matrix: None,
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
            "ed-sentry",
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
        let cli = Cli::try_parse_from(["ed-sentry", "--replay"]).unwrap();
        let error = build_runtime_command(cli).unwrap_err();

        assert!(error.message.contains("replay requires --set-file"));
    }

    #[test]
    fn cli_config_replay_rejects_journal_folder() {
        let cli = Cli::try_parse_from([
            "ed-sentry",
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
