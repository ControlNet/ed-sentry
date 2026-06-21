import { mockDashboardAdapter } from "@/adapters/mock"
import { createDefaultTauriDashboardAdapter } from "@/adapters/tauri"
import type { DashboardAdapter } from "@/adapters/types"
import { createWebDashboardAdapter } from "@/adapters/web"

export { mockDashboardAdapter } from "@/adapters/mock"

export { createTauriDashboardAdapter } from "@/adapters/tauri"
export type {
  AdapterMode,
  AppSnapshot,
  ConfigApiView,
  ConnectionStatus,
  DashboardAdapter,
  DashboardAdapterEvent,
  DashboardAdapterUnsubscribe,
  DashboardConnectionState,
  EditableConfigUpdate,
  EventFeedItem,
  JournalSourceView,
  MissionListView,
  MissionProgressView,
  MissionView,
  NotificationView,
  RateView,
  ServiceStatusKind,
  ServiceStatusView,
  SessionView,
  ValueDisplayNumber,
} from "@/adapters/types"
export { createWebDashboardAdapter } from "@/adapters/web"

export function createDashboardAdapter(): DashboardAdapter {
  const requestedMode = import.meta.env.VITE_DASHBOARD_ADAPTER
  const mode =
    requestedMode ??
    (Reflect.has(globalThis, "__TAURI_INTERNALS__") ? "tauri" : undefined) ??
    (import.meta.env.PROD ? "web" : "mock")

  switch (mode) {
    case "tauri":
      return createDefaultTauriDashboardAdapter()
    case "web": {
      const baseUrl = import.meta.env.VITE_ED_SENTRY_API_BASE_URL
      if (baseUrl === undefined) {
        return createWebDashboardAdapter()
      }
      return createWebDashboardAdapter({ baseUrl })
    }
    case "mock":
      return mockDashboardAdapter
    default:
      throw new Error(`Unsupported dashboard adapter mode: ${mode}`)
  }
}
