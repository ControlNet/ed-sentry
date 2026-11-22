import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"
import type { DashboardAdapter } from "@/adapters/dashboard"
import { loadMockConfigView, saveMockConfigView } from "@/adapters/mock-config"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { createTauriDashboardAdapter } from "@/adapters/tauri"
import { DashboardShell } from "@/components/dashboard/dashboard-shell"
import { createDashboardStore } from "./dashboard-store"

describe("desktop startup error recovery", () => {
  it("surfaces the safe startup reason, does not auto-call retryStartup, and retries on user action", async () => {
    vi.useFakeTimers()
    try {
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
      expect(retryCalls).toBe(0)
      expect(store.getState().status).toBe("error")
      expect(store.getState().errorMessage).toBe(startupFailure)

      await vi.advanceTimersByTimeAsync(5_000)

      expect(retryCalls).toBe(0)
      expect(loadCalls).toBe(1)
      expect(store.getState().status).toBe("error")

      await store.getState().retry()

      expect(retryCalls).toBe(1)
      expect(loadCalls).toBe(1)
      expect(store.getState().status).toBe("ready")
      expect(store.getState().errorMessage).toBeNull()
      expect(store.getState().snapshot?.session.commander).toBe(
        mockDashboardSnapshot.session.commander,
      )
      store.getState().unsubscribe?.()
    } finally {
      vi.useRealTimers()
    }
  })

  it("auto-recovers in the web adapter without overlapping retries and stops after unsubscribe", async () => {
    vi.useFakeTimers()
    try {
      let loadCalls = 0
      let activeRequests = 0
      let maxConcurrentRequests = 0
      const slowAttempt = createDeferred<typeof mockDashboardSnapshot>()

      const webAdapter: DashboardAdapter = {
        mode: "web",
        label: "Web API",
        async loadSnapshot() {
          loadCalls += 1
          activeRequests += 1
          maxConcurrentRequests = Math.max(maxConcurrentRequests, activeRequests)
          try {
            if (loadCalls === 1) {
              throw new Error("Web API failed: backend starting")
            }
            if (loadCalls === 2) {
              return await slowAttempt.promise
            }
            return mockDashboardSnapshot
          } finally {
            activeRequests -= 1
          }
        },
        async loadConfig() {
          return loadMockConfigView("default")
        },
        async saveConfig(update) {
          return saveMockConfigView(update)
        },
        subscribe() {
          return () => undefined
        },
      }

      const store = createDashboardStore(webAdapter)
      await store.getState().start()

      expect(loadCalls).toBe(1)
      expect(store.getState().status).toBe("error")
      expect(store.getState().snapshot).toBeNull()
      expect(store.getState().errorMessage).toBe("Web API failed: backend starting")

      // Trigger first background retry (loadCalls === 2), then advance several ticks while it is still pending.
      await vi.advanceTimersByTimeAsync(3_500)
      expect(loadCalls).toBe(2)
      expect(maxConcurrentRequests).toBe(1)
      expect(store.getState().status).toBe("error")

      slowAttempt.resolve(mockDashboardSnapshot)
      await vi.advanceTimersByTimeAsync(0)

      expect(store.getState().status).toBe("ready")
      expect(store.getState().errorMessage).toBeNull()
      expect(store.getState().connection.status).toBe("connected")
      expect(store.getState().snapshot?.session.commander).toBe(
        mockDashboardSnapshot.session.commander,
      )

      const callsBeforeUnsubscribe = loadCalls
      store.getState().unsubscribe?.()
      await vi.advanceTimersByTimeAsync(5_000)
      expect(loadCalls).toBe(callsBeforeUnsubscribe)
    } finally {
      vi.useRealTimers()
    }
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

function createDeferred<T>(): {
  readonly promise: Promise<T>
  readonly resolve: (value: T) => void
} {
  let resolvePromise: (value: T) => void = () => undefined
  const promise = new Promise<T>((resolve) => {
    resolvePromise = resolve
  })
  return {
    promise,
    resolve: (value) => resolvePromise(value),
  }
}
