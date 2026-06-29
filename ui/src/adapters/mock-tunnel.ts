import type { TunnelLoginResult, TunnelStatusView } from "@/adapters/types"
import { mockDashboardSnapshot } from "./mock-data"
import { tunnelRunningMockDashboardSnapshot } from "./mock-scenarios"

let mockTunnelAuthUnlocked = false

export function mockConfigBlockedByTunnelAuth(scenario: string): boolean {
  return scenario === "tunnel_auth_required" && !mockTunnelAuthUnlocked
}

export function mockLoadTunnelStatus(scenario: string): TunnelStatusView {
  return scenario === "tunnel_running"
    ? tunnelRunningMockDashboardSnapshot.tunnel
    : mockDashboardSnapshot.tunnel
}

export function mockStartTunnel(): TunnelStatusView {
  return tunnelRunningMockDashboardSnapshot.tunnel
}

export function mockLoginTunnel(password: string): TunnelLoginResult {
  if (password.trim().length === 0) {
    return {
      ok: false,
      code: "tunnel_login_rejected",
      message: "Tunnel login credentials were rejected",
    }
  }
  mockTunnelAuthUnlocked = true
  return { ok: true }
}
