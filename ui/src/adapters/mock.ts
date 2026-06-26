import { mockDashboardSnapshot } from "@/adapters/mock-data"
import {
  emptyMockDashboardSnapshot,
  longFeedMockDashboardSnapshot,
  privatePathMockDashboardSnapshot,
  serviceStatusesMockDashboardSnapshot,
  webLanUrlMockDashboardSnapshot,
  webUrlMockDashboardSnapshot,
} from "@/adapters/mock-scenarios"
import {
  type ConfigApiView,
  type DashboardAdapter,
  DashboardAdapterError,
  type EditableConfigUpdate,
} from "@/adapters/types"

const mockScenarioValues = [
  "default",
  "empty",
  "error",
  "loading",
  "long_feed",
  "private_path",
  "service_statuses",
  "web_lan_url",
  "web_url",
  "degraded_connection",
] as const

type MockScenario = (typeof mockScenarioValues)[number]

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
      user_id: "@bot:matrix.invalid",
      room_id: "!room:matrix.invalid",
      mention_user_id: "@operator:matrix.invalid",
      status_update_interval_seconds: 60,
      access_token_present: true,
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

export const mockDashboardAdapter: DashboardAdapter = {
  mode: "mock",
  label: "Mock live",
  async loadSnapshot() {
    const scenario = readMockScenario()
    switch (scenario) {
      case "empty":
        return emptyMockDashboardSnapshot
      case "error":
        throw new DashboardAdapterError("mock", "Test fixture dashboard load failed")
      case "loading":
        return new Promise(() => undefined)
      case "long_feed":
        return longFeedMockDashboardSnapshot
      case "private_path":
        return privatePathMockDashboardSnapshot
      case "service_statuses":
        return serviceStatusesMockDashboardSnapshot
      case "web_lan_url":
        return webLanUrlMockDashboardSnapshot
      case "web_url":
        return webUrlMockDashboardSnapshot
      case "degraded_connection":
      case "default":
        return mockDashboardSnapshot
      default:
        return assertNever(scenario)
    }
  },
  async loadConfig() {
    return mockConfigView
  },
  async saveConfig(update: EditableConfigUpdate) {
    mockConfigView = {
      ...mockConfigView,
      config: {
        journal: update.journal,
        monitor: update.monitor,
        log_levels: update.log_levels,
        matrix: {
          enabled: update.matrix.enabled,
          homeserver: update.matrix.homeserver,
          user_id: update.matrix.user_id,
          room_id: update.matrix.room_id,
          mention_user_id: update.matrix.mention_user_id,
          status_update_interval_seconds: update.matrix.status_update_interval_seconds,
          access_token_present: update.matrix.clear_access_token
            ? false
            : update.matrix.access_token_replacement !== null ||
              Boolean(mockConfigView.config.matrix?.access_token_present),
        },
        web: {
          ...update.web,
          status_label: update.web.enabled ? "Enabled" : "Disabled",
        },
      },
    }
    return mockConfigView
  },
  subscribe(onEvent) {
    if (readMockScenario() === "degraded_connection") {
      let active = true
      setTimeout(() => {
        if (!active) {
          return
        }
        onEvent({
          type: "connection",
          connection: {
            status: "degraded",
            label: "Mock degraded",
            detail: "Simulated live transport degradation",
            checkedAtDisplay: mockDashboardSnapshot.generated_at_display,
          },
        })
      }, 0)
      return () => {
        active = false
      }
    }

    onEvent({
      type: "connection",
      connection: {
        status: "connected",
        label: "Mock live",
        detail: "Using sanitized fixture-like dashboard data",
        checkedAtDisplay: mockDashboardSnapshot.generated_at_display,
      },
    })

    return () => undefined
  },
}

function readMockScenario(): MockScenario {
  if (typeof globalThis.location === "undefined") {
    return "default"
  }
  const queryValue = new URL(globalThis.location.href).searchParams.get("mock_state")
  if (queryValue === null) {
    return "default"
  }
  for (const scenario of mockScenarioValues) {
    if (queryValue === scenario) {
      return scenario
    }
  }
  return "default"
}

function assertNever(value: never): never {
  throw new Error(`Unhandled mock dashboard scenario: ${String(value)}`)
}
