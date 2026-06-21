import type { EditableConfigUpdate } from "@/adapters/dashboard"

export function mockConfigView() {
  return {
    version: 1,
    config: {
      journal: {
        folder: "/tmp/ed-sentry-fixture",
        recent_files: 5,
      },
      monitor: {
        use_utc: false,
        live_status: true,
        dynamic_title: true,
        warn_kill_rate: 20,
        warn_kill_rate_delay_minutes: 5,
        warn_no_kills_minutes: 20,
        warn_no_kills_initial_minutes: 5,
        warn_cooldown_minutes: 30,
        duplicate_max: 5,
        pirate_names: false,
        bounty_faction: false,
        bounty_value: false,
        extended_stats: false,
        min_scan_level: 1,
        poll_interval_ms: 1000,
      },
      log_levels: {
        scan_incoming: 1,
        scan_easy: 1,
        scan_hard: 1,
        kill_easy: 1,
        kill_hard: 1,
        fighter_hull: 1,
        fighter_down: 2,
        ship_shields: 1,
        ship_hull: 1,
        died: 2,
        cargo_lost: 2,
        bait_value_low: 1,
        security_scan: 1,
        security_attack: 1,
        fuel_report: 1,
        fuel_low: 2,
        fuel_critical: 2,
        missions: 1,
        missions_all: 2,
        merits: 0,
        rank_promotion: 2,
        no_kills: 2,
        kill_rate: 1,
        summary_kills: 1,
        summary_faction: 0,
        summary_scans: 0,
        summary_bounties: 1,
        summary_merits: 1,
        duplicate_suppression: 1,
      },
      matrix: {
        enabled: true,
        homeserver: "https://matrix.fixture.invalid",
        user_id: "@fixture:matrix.fixture.invalid",
        room_id: "!fixture:matrix.fixture.invalid",
        mention_user_id: null,
        status_update_interval_seconds: 60,
        access_token_present: true,
      },
      web: {
        enabled: false,
        host: "127.0.0.1",
        port: 0,
        open_browser: false,
        status_label: "Disabled",
      },
    },
    policy: {
      state_changing_enabled: true,
      state_changing_reason: "enabled for desktop config file",
      remote_bind: false,
    },
  }
}

export function mockConfigUpdate(): EditableConfigUpdate {
  const { config } = mockConfigView()
  if (config.matrix === null || config.matrix === undefined) {
    throw new Error("Mock config is missing Matrix settings")
  }

  return {
    journal: config.journal,
    monitor: config.monitor,
    log_levels: { ...config.log_levels },
    matrix: {
      enabled: config.matrix.enabled,
      homeserver: config.matrix.homeserver ?? null,
      user_id: config.matrix.user_id ?? null,
      room_id: config.matrix.room_id ?? null,
      mention_user_id: config.matrix.mention_user_id ?? null,
      status_update_interval_seconds: config.matrix.status_update_interval_seconds,
      access_token_replacement: null,
      clear_access_token: false,
    },
    web: {
      enabled: config.web.enabled,
      host: config.web.host,
      port: config.web.port,
      open_browser: config.web.open_browser,
    },
  }
}
