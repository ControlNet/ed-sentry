import { mockDashboardSnapshot } from "@/adapters/mock-data"
import {
  afkChecklistUnknownMockDashboardSnapshot,
  emptyMockDashboardSnapshot,
  longFeedMockDashboardSnapshot,
  privatePathMockDashboardSnapshot,
  serviceStatusesMockDashboardSnapshot,
  tunnelDisabledMockDashboardSnapshot,
  tunnelNonRetryableErrorMockDashboardSnapshot,
  tunnelRetryableErrorMockDashboardSnapshot,
  tunnelRunningMockDashboardSnapshot,
  webLanUrlMockDashboardSnapshot,
  webUrlMockDashboardSnapshot,
} from "@/adapters/mock-scenarios"
import {
  type DashboardAdapter,
  DashboardAdapterError,
  type EditableConfigUpdate,
} from "@/adapters/types"
import { loadMockConfigView, saveMockConfigView } from "./mock-config"
import { mockLoadTunnelStatus, mockLoginTunnel, mockStartTunnel } from "./mock-tunnel"

const mockScenarioValues = [
  "default",
  "afk_checklist_unknown",
  "empty",
  "error",
  "loading",
  "long_feed",
  "private_path",
  "service_statuses",
  "tunnel_auth_required",
  "tunnel_disabled",
  "tunnel_non_retryable_error",
  "tunnel_retryable_error",
  "tunnel_running",
  "tunnel_start",
  "web_lan_url",
  "web_url",
  "degraded_connection",
] as const

type MockScenario = (typeof mockScenarioValues)[number]

export const mockDashboardAdapter: DashboardAdapter = {
  mode: "mock",
  label: "Mock live",
  async loadSnapshot() {
    const scenario = readMockScenario()
    switch (scenario) {
      case "afk_checklist_unknown":
        return afkChecklistUnknownMockDashboardSnapshot
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
      case "tunnel_auth_required":
      case "tunnel_start":
        return mockDashboardSnapshot
      case "tunnel_disabled":
        return tunnelDisabledMockDashboardSnapshot
      case "tunnel_non_retryable_error":
        return tunnelNonRetryableErrorMockDashboardSnapshot
      case "tunnel_retryable_error":
        return tunnelRetryableErrorMockDashboardSnapshot
      case "tunnel_running":
        return tunnelRunningMockDashboardSnapshot
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
    return loadMockConfigView(readMockScenario())
  },
  async saveConfig(update: EditableConfigUpdate) {
    return saveMockConfigView(update)
  },
  async loadTunnelStatus() {
    return mockLoadTunnelStatus(readMockScenario())
  },
  async startTunnel() {
    return mockStartTunnel()
  },
  async loginTunnel(password: string) {
    return mockLoginTunnel(password)
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
