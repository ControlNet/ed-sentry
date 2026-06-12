use std::io::{self, IsTerminal, Write};

use chrono::{DateTime, Utc};
use crossterm::{
    cursor::MoveToColumn,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::config::MonitorConfig;
use crate::notifier::{AlertLevel, Notification, Notifier};
use crate::state::SessionState;
use crate::text::{format_rate_per_hour, line_safe};
use crate::time::TimeDisplayZone;

const DEFAULT_WINDOW_TITLE: &str = "ED AFK Monitor v260421";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalMode {
    Plain,
    Tty,
}

#[derive(Debug)]
pub struct TerminalNotifier<W> {
    writer: W,
    zone: TimeDisplayZone,
    mode: TerminalMode,
}

impl<W: Write> TerminalNotifier<W> {
    pub fn new(writer: W, zone: TimeDisplayZone, mode: TerminalMode) -> Self {
        Self { writer, zone, mode }
    }

    pub fn plain(writer: W, zone: TimeDisplayZone) -> Self {
        Self::new(writer, zone, TerminalMode::Plain)
    }

    pub fn tty(writer: W, zone: TimeDisplayZone) -> Self {
        Self::new(writer, zone, TerminalMode::Tty)
    }

    pub fn render_status(&mut self, status_line: &str) -> anyhow::Result<()> {
        let status_line = line_safe(status_line);
        match self.mode {
            TerminalMode::Plain => writeln!(self.writer, "{status_line}")?,
            TerminalMode::Tty => execute!(
                self.writer,
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Cyan),
                Print(status_line),
                ResetColor
            )?,
        }
        Ok(())
    }

    pub const fn supports_status_line(&self) -> bool {
        matches!(self.mode, TerminalMode::Tty)
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    fn write_notification(&mut self, notification: &Notification) -> anyhow::Result<()> {
        let line = render_notification_line(notification, self.zone);
        match self.mode {
            TerminalMode::Plain => writeln!(self.writer, "{line}")?,
            TerminalMode::Tty => execute!(
                self.writer,
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(alert_color(notification.alert_level)),
                Print(line),
                ResetColor,
                Print("\n")
            )?,
        }
        Ok(())
    }
}

impl TerminalNotifier<io::Stdout> {
    pub fn stdout(config: &MonitorConfig) -> Self {
        let stdout = io::stdout();
        let mode = if config.live_status && stdout.is_terminal() {
            TerminalMode::Tty
        } else {
            TerminalMode::Plain
        };
        let zone = if config.use_utc {
            TimeDisplayZone::Utc
        } else {
            TimeDisplayZone::Local
        };
        Self::new(stdout, zone, mode)
    }
}

impl<W: Write> Notifier for TerminalNotifier<W> {
    fn send(&mut self, notification: &Notification) -> anyhow::Result<()> {
        self.write_notification(notification)
    }
}

pub fn render_notification_line(notification: &Notification, zone: TimeDisplayZone) -> String {
    render_log_line(
        notification.timestamp,
        zone,
        notification.emoji.as_deref(),
        &notification.terminal_text,
    )
}

pub fn render_log_line(
    timestamp: DateTime<Utc>,
    zone: TimeDisplayZone,
    emoji: Option<&str>,
    text: &str,
) -> String {
    let timestamp = format_log_timestamp(timestamp, zone);
    let text = log_text_safe(text);
    match emoji.map(line_safe) {
        Some(emoji) if !emoji.trim().is_empty() => format!("{timestamp}{emoji} {text}"),
        _ => format!("{timestamp}{text}"),
    }
}

fn log_text_safe(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' => ' ',
            '\n' => '\n',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect()
}

pub fn format_log_timestamp(timestamp: DateTime<Utc>, zone: TimeDisplayZone) -> String {
    let formatted = match zone {
        TimeDisplayZone::Utc => timestamp.format("%H:%M:%S").to_string(),
        TimeDisplayZone::Local => timestamp
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S")
            .to_string(),
        TimeDisplayZone::FixedOffset(offset) => timestamp
            .with_timezone(&offset)
            .format("%H:%M:%S")
            .to_string(),
    };
    format!("[{formatted}]")
}

pub fn render_banner(title: &str) -> String {
    let title = line_safe(title);
    let rule = "=".repeat(title.chars().count());
    format!("{rule}\n{title}\n{rule}")
}

pub fn render_status_line(
    state: &SessionState,
    config: &MonitorConfig,
    now: DateTime<Utc>,
) -> String {
    render_monitor_status_line(
        state,
        config,
        now,
        state.mission_completed,
        state.mission_total as usize,
    )
}

pub fn render_monitor_status_line(
    state: &SessionState,
    config: &MonitorConfig,
    now: DateTime<Utc>,
    mission_completed: u64,
    active_missions: usize,
) -> String {
    let kill_rate = match state.kills {
        0 => "-/h".to_string(),
        _ => format_rate_per_hour(state.total_kill_rate_per_hour_at(now)),
    };
    let last_kill = state
        .last_kill_at
        .map(|last_kill_at| status_duration(now.signed_duration_since(last_kill_at)))
        .unwrap_or_else(|| status_duration(session_elapsed(state, now)));
    let scan_rate = match state.cargo_scans {
        0 => "-/h".to_string(),
        _ => format_rate_per_hour(state.total_scan_rate_per_hour_at(now)),
    };
    let last_scan = state
        .last_scan_at
        .map(|last_scan_at| status_duration(now.signed_duration_since(last_scan_at)))
        .unwrap_or_else(|| status_duration(session_elapsed(state, now)));
    let session_elapsed = status_duration(session_elapsed(state, now));
    let kills = format!("{kill_rate} (+{last_kill}) [x{}]", state.kills);
    let scans = format!("{scan_rate} (+{last_scan}) [x{}]", state.cargo_scans);

    format!(
        "{}💥 {kills:<23} | 📦 {scans:<23} | ⏱️ {session_elapsed:<5} | 🎯 {mission_completed}/{active_missions}",
        format_log_timestamp(
            now,
            if config.use_utc {
                TimeDisplayZone::Utc
            } else {
                TimeDisplayZone::Local
            }
        )
    )
}

pub fn render_dynamic_title(
    state: &SessionState,
    _config: &MonitorConfig,
    now: DateTime<Utc>,
    mission_completed: u64,
    active_missions: usize,
) -> String {
    if !state.active_session {
        return DEFAULT_WINDOW_TITLE.to_string();
    }

    let kill_rate = match state.kills {
        0 => "-".to_string(),
        _ => format_rate_per_hour(state.total_kill_rate_per_hour_at(now))
            .trim_end_matches("/h")
            .to_string(),
    };
    let last_kill = state
        .last_kill_at
        .map(|last_kill_at| status_duration(now.signed_duration_since(last_kill_at)))
        .unwrap_or_else(|| status_duration(session_elapsed(state, now)));
    format!(
        "💥{kill_rate}/h ⌚{last_kill} 🎯 {}/{}",
        mission_completed, active_missions
    )
}

pub fn set_platform_window_title(_title: &str) {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::System::Console::SetConsoleTitleW;

        let wide = OsStr::new(_title)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        unsafe {
            SetConsoleTitleW(wide.as_ptr());
        }
    }
}

fn session_elapsed(state: &SessionState, now: DateTime<Utc>) -> chrono::Duration {
    state
        .session_started_at
        .map(|started_at| now.signed_duration_since(started_at))
        .unwrap_or_else(chrono::Duration::zero)
}

fn status_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds().max(0);
    if total_seconds < 60 {
        return format!("{total_seconds}s");
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}h{minutes}m")
    } else {
        format!("{minutes}m{seconds}s")
    }
}

fn alert_color(alert_level: AlertLevel) -> Color {
    match alert_level {
        AlertLevel::Info => Color::White,
        AlertLevel::Warn => Color::Yellow,
        AlertLevel::Critical => Color::Red,
    }
}
