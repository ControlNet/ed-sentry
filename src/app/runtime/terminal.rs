use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::app::{MatrixStartupStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::delivery::DeliveryHub;
use crate::event::{parse_journal_line, JournalEvent};
use crate::journal::{
    configured_recent_journal_file_choices, select_configured_journal_file, stream_journal_file,
};
use crate::monitor::EventMonitor;
use crate::notifier::Notification;
use crate::terminal::TerminalNotifier;
use crate::text::line_safe;

use super::watch_runner::{self, TitleMode};
use super::{
    build_watch_delivery, journal_basename, matrix_startup_label, selected_journal_from_choices,
    start_webui, watch_journal_folder_display, JournalSelector, MonitorRuntime, RuntimeError,
};

pub async fn run_watch(config: &RuntimeConfig) -> Result<(), RuntimeError> {
    let program_started_at = Utc::now();
    let mut selector = TerminalJournalSelector;
    let mut runtime = MonitorRuntime::start(
        config,
        &mut selector,
        MatrixStartupStatus::from_runtime_config(config),
        WebStartupStatus::from_current_runtime_config(config),
    )?;
    let startup = runtime.startup().clone();
    print_startup(config, &startup.journal_file, startup.commander.as_deref());
    let _web_server = start_webui(config, &mut runtime).await;
    let super::WatchDelivery {
        hub: mut delivery,
        matrix_status,
    } = build_watch_delivery(config).await;
    runtime.set_matrix_status(matrix_status.clone());
    print_matrix_startup_status(&matrix_status);
    watch_runner::send_startup_header(&mut delivery, config, &runtime, program_started_at).await?;
    if config.debug {
        eprintln!(
            "Debug: preloaded {} lines from {} to byte offset {}",
            startup.preload_line_count,
            line_safe(&startup.journal_file.display().to_string()),
            startup.preload_eof_offset
        );
    }
    watch_runner::run_startup(
        &mut runtime,
        &mut delivery,
        config,
        program_started_at,
        TitleMode::PlatformWindow,
    )
    .await?;

    watch_loop::run(&mut runtime, &mut delivery, &startup.journal_file).await
}

pub async fn run_replay(config: &RuntimeConfig) -> Result<(), RuntimeError> {
    let set_file = select_configured_journal_file(config).map_err(RuntimeError::from)?;
    if config.debug {
        eprintln!(
            "Debug: replaying Journal file {}",
            line_safe(&set_file.display().to_string())
        );
    }
    let scan = scan_replay_startup(&set_file)?;
    print_startup(config, &set_file, scan.commander.as_deref());
    if config.debug {
        eprintln!(
            "Debug: replay loaded {} lines to byte offset {}",
            scan.line_count, scan.eof_offset
        );
    }

    let mut delivery = DeliveryHub::terminal_only(TerminalNotifier::stdout(&config.monitor));
    let notifications = replay_notifications(config, &set_file)?;
    deliver_replay_notifications(&mut delivery, &notifications).await
}

struct TerminalJournalSelector;

mod watch_loop;

#[cfg(test)]
mod tests;

impl JournalSelector for TerminalJournalSelector {
    fn select(&mut self, config: &RuntimeConfig) -> Result<PathBuf, RuntimeError> {
        if config.set_file.is_some() || !config.file_select {
            return select_configured_journal_file(config).map_err(RuntimeError::from);
        }

        select_interactive_journal_file(config)
    }
}

struct ReplayStartupScan {
    line_count: usize,
    eof_offset: u64,
    commander: Option<String>,
}

fn select_interactive_journal_file(config: &RuntimeConfig) -> Result<PathBuf, RuntimeError> {
    let choices = configured_recent_journal_file_choices(config).map_err(RuntimeError::from)?;
    for choice in &choices {
        println!(
            "{}: {}",
            choice.number,
            line_safe(&choice.file.path.display().to_string())
        );
    }
    print!("Select Journal file [1-{}]: ", choices.len());
    io::stdout()
        .flush()
        .map_err(|error| RuntimeError::new(error.to_string()))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    let selected = input
        .trim()
        .parse::<usize>()
        .map_err(|_| RuntimeError::new(format!("invalid Journal selection: {}", input.trim())))?;

    selected_journal_from_choices(choices, selected)
}

fn scan_replay_startup(set_file: &Path) -> Result<ReplayStartupScan, RuntimeError> {
    let mut line_count = 0_usize;
    let mut commander = None;
    let scan = stream_journal_file(set_file, parse_journal_line, |record| {
        line_count += 1;
        if commander.is_none() {
            commander = match record.result.as_ref().ok() {
                Some(JournalEvent::Commander(event)) => event.name.clone(),
                Some(JournalEvent::LoadGame(event)) => event.commander.clone(),
                _ => None,
            };
        }
        Ok::<(), std::convert::Infallible>(())
    })
    .map_err(|error| RuntimeError::new(error.to_string()))?;

    Ok(ReplayStartupScan {
        line_count,
        eof_offset: scan.eof_offset,
        commander,
    })
}

fn replay_notifications(
    config: &RuntimeConfig,
    set_file: &Path,
) -> Result<Vec<Notification>, RuntimeError> {
    let mut monitor = EventMonitor::new(config.monitor.clone(), config.log_levels.clone());
    let mut last_timestamp = None;
    let mut notifications = Vec::new();

    stream_journal_file(set_file, parse_journal_line, |record| {
        match record.result {
            Ok(event) => {
                last_timestamp = Some(event.timestamp());
                notifications.extend(monitor.process_event(&event));
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
    .map_err(|error| RuntimeError::new(error.to_string()))?;

    if let Some(timestamp) = last_timestamp {
        notifications.push(monitor.start_monitor(&journal_basename(set_file), timestamp));
        notifications.extend(monitor.finish(&journal_basename(set_file), timestamp));
    }
    Ok(notifications)
}

async fn deliver_replay_notifications(
    delivery: &mut DeliveryHub<std::io::Stdout>,
    notifications: &[Notification],
) -> Result<(), RuntimeError> {
    let warnings = delivery
        .send_notifications(notifications)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    watch_runner::print_delivery_warnings(warnings);
    Ok(())
}

fn print_startup(config: &RuntimeConfig, set_file: &Path, commander: Option<&str>) {
    println!(
        "Journal folder: {}",
        line_safe(&watch_journal_folder_display(config))
    );
    println!("Journal file: {}", line_safe(&journal_basename(set_file)));
    println!(
        "Commander name: {}",
        line_safe(commander.unwrap_or("[Unknown]"))
    );
    println!("Config profile: Default");
    println!("\nStarting... (Press Ctrl+C to stop)\n");
}

fn print_matrix_startup_status(status: &MatrixStartupStatus) {
    match matrix_startup_label(status) {
        "disabled" => {
            println!("Info: Matrix delivery disabled - operating with terminal output only\n");
        }
        "enabled" => {
            println!("Info: Matrix delivery enabled\n");
        }
        "unavailable" => {
            println!("Info: Matrix delivery unavailable - operating with terminal output only\n");
        }
        _ => {}
    }
}
