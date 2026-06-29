import type { ConfigApiView, EditableConfigUpdate } from "@/adapters/types"
import { DashboardAdapterError } from "@/adapters/types"
import { mockConfigBlockedByTunnelAuth } from "./mock-tunnel"

let mockConfigView: ConfigApiView = {
  version: 1,
  config: {
    journal: {
      folder: "Sanitized Journal source",
      recent_files: 3,
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
      homeserver: "https://matrix.invalid",
      room_id: "!room:matrix.invalid",
      mention_user_id: "@operator:matrix.invalid",
      status_update_interval_seconds: 60,
      access_token_present: true,
    },
    tunnel: {
      provider: "cloudflare_quick",
      auto_start: false,
      config_password_present: false,
    },
    web: {
      enabled: true,
      host: "127.0.0.1",
      port: 4173,
      open_browser: false,
      status_label: "Enabled",
    },
  },
  policy: {
    state_changing_enabled: true,
    state_changing_reason: "enabled for trusted WebUI clients",
    remote_bind: false,
  },
}

export function loadMockConfigView(scenario: string): ConfigApiView {
  if (mockConfigBlockedByTunnelAuth(scenario)) {
    throw new DashboardAdapterError("mock", "Tunnel config access requires a valid bearer token")
  }
  return mockConfigView
}

export function saveMockConfigView(update: EditableConfigUpdate): ConfigApiView {
  mockConfigView = {
    ...mockConfigView,
    config: {
      journal: update.journal,
      monitor: update.monitor,
      log_levels: update.log_levels,
      matrix: {
        enabled: update.matrix.enabled,
        homeserver: update.matrix.homeserver,
        room_id: update.matrix.room_id,
        mention_user_id: update.matrix.mention_user_id,
        status_update_interval_seconds: update.matrix.status_update_interval_seconds,
        access_token_present: update.matrix.clear_access_token
          ? false
          : update.matrix.access_token_replacement !== null ||
            Boolean(mockConfigView.config.matrix?.access_token_present),
      },
      tunnel: {
        provider: update.tunnel.provider,
        auto_start: update.tunnel.auto_start,
        config_password_present: update.tunnel.clear_config_password
          ? false
          : update.tunnel.config_password_replacement !== null ||
            mockConfigView.config.tunnel.config_password_present,
      },
      web: {
        ...update.web,
        status_label: update.web.enabled ? "Enabled" : "Disabled",
      },
    },
  }
  return mockConfigView
}
