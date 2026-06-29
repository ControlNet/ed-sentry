import { expect, test } from "@playwright/test"
import { createTauriDashboardAdapter, createWebDashboardAdapter } from "@/adapters/dashboard"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import type { EditableConfigUpdate } from "@/adapters/types"
import { mockConfigUpdate, mockConfigView, mockTunnelStatusView } from "./adapter-boundary-fixtures"

const tunnelAuthStorageKey = "ed-sentry:tunnel-auth-token"

class FakeSessionStorage implements Storage {
  private readonly values = new Map<string, string>()

  get length(): number {
    return this.values.size
  }

  clear(): void {
    this.values.clear()
  }

  getItem(key: string): string | null {
    return this.values.get(key) ?? null
  }

  key(index: number): string | null {
    return Array.from(this.values.keys())[index] ?? null
  }

  removeItem(key: string): void {
    this.values.delete(key)
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value)
  }
}

test("web adapter stores tunnel login token and sends Bearer only to protected config", async () => {
  const originalFetch = globalThis.fetch
  const originalSessionStorage = globalThis.sessionStorage
  const sessionStorage = new FakeSessionStorage()
  const requestLog: Request[] = []

  Object.defineProperty(globalThis, "sessionStorage", { configurable: true, value: sessionStorage })
  Object.defineProperty(globalThis, "fetch", {
    configurable: true,
    value: async (input: Parameters<typeof fetch>[0], init: Parameters<typeof fetch>[1]) => {
      const request = new Request(input, init)
      requestLog.push(request)
      const requestedPath = new URL(request.url).pathname
      if (requestedPath === "/api/tunnel/status") {
        return jsonResponse(runningTunnelStatus("session-a"))
      }
      if (requestedPath === "/api/tunnel/login") {
        return jsonResponse({ token: "x" })
      }
      if (requestedPath === "/api/config") {
        return jsonResponse(mockConfigView())
      }
      if (requestedPath === "/api/snapshot") {
        return jsonResponse(mockDashboardSnapshot)
      }
      return new Response("unexpected endpoint", { status: 500 })
    },
  })

  try {
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    if (adapter.loginTunnel === undefined || adapter.loadTunnelStatus === undefined) {
      throw new Error("Web adapter did not expose tunnel login/status methods")
    }

    await expect(adapter.loadTunnelStatus()).resolves.toMatchObject({ kind: "running" })
    await expect(adapter.loginTunnel("fixture password")).resolves.toEqual({ ok: true })
    await expect(adapter.loadSnapshot()).resolves.toEqual(mockDashboardSnapshot)
    await expect(adapter.loadConfig()).resolves.toEqual(mockConfigView())
  } finally {
    restoreGlobal("fetch", originalFetch)
    restoreGlobal("sessionStorage", originalSessionStorage)
  }

  expect(sessionStorage.getItem(tunnelAuthStorageKey)).not.toBeNull()
  expect(
    requestLog.map((request) => ({
      path: new URL(request.url).pathname,
      authorization: request.headers.get("authorization"),
    })),
  ).toEqual([
    { path: "/api/tunnel/status", authorization: null },
    { path: "/api/tunnel/login", authorization: null },
    { path: "/api/tunnel/status", authorization: null },
    { path: "/api/snapshot", authorization: null },
    { path: "/api/tunnel/status", authorization: null },
    { path: "/api/config", authorization: ["Bearer", "x"].join(" ") },
  ])
})

test("web adapter clears tunnel token on login failure and protected auth failure", async () => {
  const originalFetch = globalThis.fetch
  const originalSessionStorage = globalThis.sessionStorage
  const sessionStorage = new FakeSessionStorage()
  let loginSucceeds = false

  Object.defineProperty(globalThis, "sessionStorage", { configurable: true, value: sessionStorage })
  Object.defineProperty(globalThis, "fetch", {
    configurable: true,
    value: async (input: Parameters<typeof fetch>[0], init: Parameters<typeof fetch>[1]) => {
      const request = new Request(input, init)
      const requestedPath = new URL(request.url).pathname
      if (requestedPath === "/api/tunnel/status") {
        return jsonResponse(runningTunnelStatus("session-a"))
      }
      if (requestedPath === "/api/tunnel/login") {
        if (loginSucceeds) {
          return jsonResponse({ token: "x" })
        }
        return jsonResponse(loginRejectedBody("Rejected"), { status: 403 })
      }
      if (requestedPath === "/api/config") {
        return jsonResponse(loginRejectedBody("Forbidden"), { status: 403 })
      }
      return new Response("unexpected endpoint", { status: 500 })
    },
  })

  try {
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    if (adapter.loginTunnel === undefined) {
      throw new Error("Web adapter did not expose tunnel login")
    }

    sessionStorage.setItem(tunnelAuthStorageKey, "stale")
    await expect(adapter.loginTunnel("bad password")).resolves.toEqual({
      ok: false,
      code: "tunnel_login_rejected",
      message: "Rejected",
    })
    expect(sessionStorage.getItem(tunnelAuthStorageKey)).toBeNull()

    loginSucceeds = true
    await expect(adapter.loginTunnel("fixture password")).resolves.toEqual({ ok: true })
    expect(sessionStorage.getItem(tunnelAuthStorageKey)).not.toBeNull()
    await expect(adapter.loadConfig()).rejects.toThrow(/Forbidden/)
    expect(sessionStorage.getItem(tunnelAuthStorageKey)).toBeNull()
  } finally {
    restoreGlobal("fetch", originalFetch)
    restoreGlobal("sessionStorage", originalSessionStorage)
  }
})

test("web adapter clears stale tunnel token when status host or session changes", async () => {
  const originalFetch = globalThis.fetch
  const originalSessionStorage = globalThis.sessionStorage
  const sessionStorage = new FakeSessionStorage()
  const configAuthorizations: (string | null)[] = []

  Object.defineProperty(globalThis, "sessionStorage", { configurable: true, value: sessionStorage })
  Object.defineProperty(globalThis, "fetch", {
    configurable: true,
    value: async (input: Parameters<typeof fetch>[0], init: Parameters<typeof fetch>[1]) => {
      const request = new Request(input, init)
      const requestedPath = new URL(request.url).pathname
      if (requestedPath === "/api/tunnel/status") {
        return jsonResponse(runningTunnelStatus("session-b"))
      }
      if (requestedPath === "/api/config") {
        configAuthorizations.push(request.headers.get("authorization"))
        return jsonResponse(mockConfigView())
      }
      return new Response("unexpected endpoint", { status: 500 })
    },
  })

  try {
    sessionStorage.setItem(
      tunnelAuthStorageKey,
      JSON.stringify({
        token: "x",
        publicHost: "session-a.trycloudflare.com",
        sessionId: "session-a",
      }),
    )
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    await expect(adapter.loadConfig()).resolves.toEqual(mockConfigView())
  } finally {
    restoreGlobal("fetch", originalFetch)
    restoreGlobal("sessionStorage", originalSessionStorage)
  }

  expect(configAuthorizations).toEqual([null])
  expect(sessionStorage.getItem(tunnelAuthStorageKey)).toBeNull()
})

test("tauri adapter forwards tunnel commands without using browser token storage", async () => {
  const tunnelStatus = mockTunnelStatusView()
  const calls: string[] = []
  const originalSessionStorage = globalThis.sessionStorage
  const sessionStorage = new FakeSessionStorage()
  Object.defineProperty(globalThis, "sessionStorage", { configurable: true, value: sessionStorage })

  try {
    const adapter = createTauriDashboardAdapter({
      loadSnapshot: async () => mockDashboardSnapshot,
      loadConfig: async () => mockConfigView(),
      saveConfig: async (_update: EditableConfigUpdate) => {
        calls.push("saveConfig")
        return mockConfigView()
      },
      loadTunnelStatus: async () => {
        calls.push("loadTunnelStatus")
        return tunnelStatus
      },
      startTunnel: async () => {
        calls.push("startTunnel")
        return { ...tunnelStatus, kind: "disabled", status_label: "Disabled" }
      },
    })

    if (adapter.loadTunnelStatus === undefined || adapter.startTunnel === undefined) {
      throw new Error("Tauri adapter did not expose tunnel command methods")
    }

    await expect(adapter.loadTunnelStatus()).resolves.toEqual(tunnelStatus)
    await expect(adapter.startTunnel()).resolves.toMatchObject({
      kind: "disabled",
      status_label: "Disabled",
    })
    await expect(adapter.saveConfig(mockConfigUpdate())).resolves.toEqual(mockConfigView())
  } finally {
    restoreGlobal("sessionStorage", originalSessionStorage)
  }

  expect(calls).toEqual(["loadTunnelStatus", "startTunnel", "saveConfig"])
  expect(sessionStorage.getItem(tunnelAuthStorageKey)).toBeNull()
})

function runningTunnelStatus(sessionId: string) {
  return {
    ...mockTunnelStatusView(),
    kind: "running",
    session_id: sessionId,
    public_url: `https://${sessionId}.trycloudflare.com`,
  }
}

function loginRejectedBody(message: string) {
  return { error: { code: "tunnel_login_rejected", message } }
}

function jsonResponse(payload: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(payload), {
    ...init,
    headers: { "content-type": "application/json", ...init.headers },
  })
}

function restoreGlobal(key: "fetch", value: typeof globalThis.fetch): void
function restoreGlobal(key: "sessionStorage", value: Storage): void
function restoreGlobal(
  key: "fetch" | "sessionStorage",
  value: typeof globalThis.fetch | Storage,
): void {
  Object.defineProperty(globalThis, key, { configurable: true, value })
}
