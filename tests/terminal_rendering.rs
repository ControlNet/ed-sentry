use chrono::{DateTime, Duration, TimeZone, Utc};
use ed_afk_monitor::config::MonitorConfig;
use ed_afk_monitor::notifier::{AlertLevel, Notification, Notifier};
use ed_afk_monitor::state::SessionState;
use ed_afk_monitor::terminal::{
    render_dynamic_title, render_status_line, TerminalMode, TerminalNotifier,
};
use ed_afk_monitor::time::TimeDisplayZone;

fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2035, 6, 9, 16, 30, 0)
        .single()
        .unwrap()
}

fn notification(text: &str) -> Notification {
    Notification::new(
        "terminal_rendering_fixture",
        2,
        AlertLevel::Warn,
        Some("!".to_string()),
        text,
        text,
        fixed_now(),
    )
}

fn fixed_status_state() -> SessionState {
    let now = fixed_now();
    let mut state = SessionState::new();
    state.active_session = true;
    state.session_started_at = Some(now - Duration::seconds(11_412));
    state.kills = 71;
    state.cargo_scans = 96;
    state.last_kill_at = Some(now - Duration::seconds(58));
    state.last_scan_at = Some(now - Duration::seconds(58));
    state.mission_completed = 5;
    state.mission_total = 20;
    state.shields_up = Some(true);
    state
}

#[test]
fn terminal_rendering_notification_logs_are_line_safe() {
    let mut notifier = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);

    notifier
        .send(&notification("Kill confirmed\r\nSecond line\x1b[2Ktail"))
        .unwrap();

    let output = String::from_utf8(notifier.into_inner()).unwrap();
    assert!(output.contains("[16:30:00]!"), "{output:?}");
    assert!(output.contains("Kill confirmed"), "{output:?}");
    assert!(output.contains("Second line"), "{output:?}");
    assert!(output.contains("[2Ktail"), "{output:?}");
    assert!(!output.contains('\r'), "{output:?}");
    assert!(!output.contains('\x1b'), "{output:?}");
    assert_eq!(output.lines().count(), 2, "{output:?}");
}

#[test]
fn terminal_status_fixed_fragments() {
    let line = render_status_line(
        &fixed_status_state(),
        &MonitorConfig::default(),
        fixed_now(),
    );

    for fragment in [
        "[16:30:00]💥 22.4/h (+58s) [x71]",
        "📦 30.3/h (+58s) [x96]",
        "⏱️ 3h10m",
        "🎯 5/20",
    ] {
        assert!(line.contains(fragment), "{line}");
    }
    assert!(!line.contains("Shield"), "{line}");
}

#[test]
fn terminal_dynamic_title_matches_upstream_active_and_reset_text() {
    let active = render_dynamic_title(
        &fixed_status_state(),
        &MonitorConfig::default(),
        fixed_now(),
        5,
        20,
    );
    assert_eq!(active, "💥22.4/h ⌚58s 🎯 5/20");

    let mut inactive = fixed_status_state();
    inactive.active_session = false;
    let reset = render_dynamic_title(&inactive, &MonitorConfig::default(), fixed_now(), 5, 20);
    assert_eq!(reset, "ED AFK Monitor v260421");
}

#[test]
fn terminal_status_missing_last_kill_and_unknown_shield_are_locked_fragments() {
    let mut state = fixed_status_state();
    state.last_kill_at = None;
    state.shields_up = None;

    let line = render_status_line(&state, &MonitorConfig::default(), fixed_now());

    assert!(line.contains("(+3h10m) [x71]"), "{line}");
    assert!(!line.contains("Shield"), "{line}");
}

#[test]
fn terminal_non_tty_plain_output_has_no_control_sequences() {
    let mut notifier = TerminalNotifier::new(Vec::new(), TimeDisplayZone::Utc, TerminalMode::Plain);

    notifier
        .send(&notification("Cargo scan\x1b[2K clean"))
        .unwrap();
    notifier
        .render_status("Kills 71\r\x1b[2K Scans 96")
        .unwrap();

    let output = String::from_utf8(notifier.into_inner()).unwrap();
    assert!(
        output.contains("[16:30:00]! Cargo scan [2K clean"),
        "{output:?}"
    );
    assert!(output.contains("Kills 71  [2K Scans 96"), "{output:?}");
    assert!(!output.contains('\x1b'), "{output:?}");
    assert!(!output.contains("\x1b[2K"), "{output:?}");
    assert!(!output.contains('\r'), "{output:?}");
}

#[test]
fn terminal_rendering_tty_status_uses_clear_current_line_sequence() {
    let mut notifier = TerminalNotifier::tty(Vec::new(), TimeDisplayZone::Utc);

    assert!(notifier.supports_status_line());
    notifier.render_status("Kills 71").unwrap();

    let output = String::from_utf8(notifier.into_inner()).unwrap();
    assert!(output.contains("\x1b[2K"), "{output:?}");
    assert!(output.contains("Kills 71"), "{output:?}");
}
