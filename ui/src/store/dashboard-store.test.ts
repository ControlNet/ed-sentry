import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"
import { loadMockConfigView, saveMockConfigView } from "@/adapters/mock-config"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { createTauriDashboardAdapter } from "@/adapters/tauri"
import { DashboardShell } from "@/components/dashboard/dashboard-shell"
import { createDashboardStore } from "./dashboard-store"

describe("desktop startup error recovery", () => {
  it("surfaces the safe startup reason and retries via retryStartup instead of cached loadSnapshot", async () => {
    let retryCalls = 0
    let loadCalls = 0
    const startupFailure =
      "Desktop monitor startup failed: no Journal.*.log files were found in the configured Journal folder"
    const adapter = createTauriDashboardAdapter({
      async loadSnapshot() {
        loadCalls += 1
        return Promise.reject(startupFailure)
      },
      async retryStartup() {
        retryCalls += 1
        return mockDashboardSnapshot
      },
      async loadConfig() {
        return loadMockConfigView("default")
      },
      async saveConfig(update) {
        return saveMockConfigView(update)
      },
    })

    const store = createDashboardStore(adapter)
    await store.getState().start()

    expect(loadCalls).toBe(1)
    expect(store.getState().status).toBe("error")
    expect(store.getState().errorMessage).toBe(startupFailure)

    await store.getState().retry()

    expect(retryCalls).toBe(1)
    expect(loadCalls).toBe(1)
    expect(store.getState().status).toBe("ready")
    expect(store.getState().snapshot?.session.commander).toBe(
      mockDashboardSnapshot.session.commander,
    )
    store.getState().unsubscribe?.()
  })

  it("renders error reason, Retry, and Systems navigation when snapshot startup fails", () => {
    const adapter = createTauriDashboardAdapter({
      loadSnapshot: async () =>
        Promise.reject(
          "Desktop monitor startup failed: configured Journal folder could not be read: not found",
        ),
      loadConfig: async () => loadMockConfigView("default"),
      saveConfig: async (update) => saveMockConfigView(update),
    })

    const errorHtml = renderToStaticMarkup(
      createElement(DashboardShell, {
        snapshot: null,
        errorMessage:
          "Desktop monitor startup failed: configured Journal folder could not be read: not found",
        adapter,
        connection: {
          status: "error",
          label: adapter.label,
          detail:
            "Desktop monitor startup failed: configured Journal folder could not be read: not found",
          checkedAtDisplay: null,
        },
        isRefreshing: false,
        onRefresh: () => undefined,
      }),
    )
    expect(errorHtml).toContain("Dashboard unavailable")
    expect(errorHtml).toContain("configured Journal folder could not be read: not found")
    expect(errorHtml).toContain("Retry")
    expect(errorHtml).toContain("Open Systems")
    expect(errorHtml).toContain('data-workspace-tab="config"')

    const systemsHtml = renderToStaticMarkup(
      createElement(DashboardShell, {
        snapshot: null,
        errorMessage:
          "Desktop monitor startup failed: configured Journal folder could not be read: not found",
        adapter,
        connection: {
          status: "error",
          label: adapter.label,
          detail:
            "Desktop monitor startup failed: configured Journal folder could not be read: not found",
          checkedAtDisplay: null,
        },
        isRefreshing: false,
        onRefresh: () => undefined,
        initialTab: "config",
      }),
    )
    expect(systemsHtml).toContain('aria-label="Config editor"')
    expect(systemsHtml).toContain("Retry")
  })
})
