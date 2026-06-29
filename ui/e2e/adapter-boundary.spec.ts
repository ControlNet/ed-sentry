import { expect, test } from "@playwright/test"
import {
  createWebDashboardAdapter,
  type DashboardAdapter,
  type DashboardAdapterEvent,
} from "@/adapters/dashboard"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { parseAppSnapshot, parseTunnelStatusView } from "@/adapters/types"
import { formFromConfig, updateFromForm } from "@/components/dashboard/config-form-model"
import { shouldApplySnapshotUpdate } from "@/store/snapshot-normalization"
import { mockConfigView, mockTunnelStatusView } from "./adapter-boundary-fixtures"

type FakeWebSocketEvent = {
  readonly data?: string
}

type FakeWebSocketEventType = "open" | "message" | "error" | "close"

type FakeWebSocketListener = (event: FakeWebSocketEvent) => void

class FakeWebSocket {
  static instance: FakeWebSocket | null = null

  readonly listeners: Record<FakeWebSocketEventType, FakeWebSocketListener[]> = {
    open: [],
    message: [],
    error: [],
    close: [],
  }

  readonly url: string

  constructor(url: string | URL) {
    this.url = String(url)
    FakeWebSocket.instance = this
  }

  addEventListener(type: FakeWebSocketEventType, listener: FakeWebSocketListener): void {
    this.listeners[type].push(listener)
  }

  close(): void {
    this.emit("close", {})
  }

  emit(type: FakeWebSocketEventType, event: FakeWebSocketEvent): void {
    for (const listener of this.listeners[type]) {
      listener(event)
    }
  }
}

test.afterEach(() => {
  FakeWebSocket.instance = null
})

test("web adapter reports malformed WebSocket JSON as a degraded connection", () => {
  const originalWebSocket = globalThis.WebSocket
  Object.defineProperty(globalThis, "WebSocket", {
    configurable: true,
    value: FakeWebSocket,
  })
  const events: DashboardAdapterEvent[] = []

  try {
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    const unsubscribe = subscribeForTest(adapter, events)
    requireFakeWebSocket().emit("message", { data: "not-json" })
    unsubscribe()
  } finally {
    Object.defineProperty(globalThis, "WebSocket", {
      configurable: true,
      value: originalWebSocket,
    })
  }

  expect(events).toContainEqual({
    type: "connection",
    connection: {
      status: "degraded",
      label: "Message ignored",
      detail: "The WebSocket message was not valid JSON",
      checkedAtDisplay: null,
    },
  })
})

test("web adapter treats hello as one bootstrap snapshot", () => {
  const originalWebSocket = globalThis.WebSocket
  Object.defineProperty(globalThis, "WebSocket", {
    configurable: true,
    value: FakeWebSocket,
  })
  const events: DashboardAdapterEvent[] = []
  const bufferedEvent = requireMockEvent("mock-event-004")

  try {
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    const unsubscribe = subscribeForTest(adapter, events)
    requireFakeWebSocket().emit("message", {
      data: JSON.stringify({
        type: "hello",
        snapshot: mockDashboardSnapshot,
        event_feed: [bufferedEvent],
      }),
    })
    unsubscribe()
  } finally {
    Object.defineProperty(globalThis, "WebSocket", {
      configurable: true,
      value: originalWebSocket,
    })
  }

  expect(events).toContainEqual({ type: "snapshot", snapshot: mockDashboardSnapshot })
  expect(events).not.toContainEqual({ type: "event", item: bufferedEvent })
  expect(events.filter((event) => event.type === "snapshot")).toHaveLength(1)
})

test("dashboard store ignores volatile-only snapshot updates", () => {
  const volatileOnlySnapshot = {
    ...mockDashboardSnapshot,
    generated_at: "2026-06-20T14:18:03Z",
    generated_at_display: "2026-06-20T14:18:03Z",
    event_feed: [...mockDashboardSnapshot.event_feed].reverse(),
  }
  const changedSnapshot = {
    ...mockDashboardSnapshot,
    session: {
      ...mockDashboardSnapshot.session,
      kills: mockDashboardSnapshot.session.kills + 1,
    },
  }

  expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, volatileOnlySnapshot)).toBe(false)
  expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, changedSnapshot)).toBe(true)
})

test("dashboard store applies afk checklist-only snapshot updates", () => {
  const checklistOnlySnapshot = {
    ...mockDashboardSnapshot,
    afk_checklist: {
      rows: mockDashboardSnapshot.afk_checklist.rows.map((row) =>
        row.id === "cargo_loaded" ? { ...row, state: "pass" as const } : row,
      ),
    },
  }

  expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, checklistOnlySnapshot)).toBe(true)
})

test("dashboard store applies tunnel-only snapshot updates", () => {
  const tunnelOnlySnapshot = {
    ...mockDashboardSnapshot,
    tunnel: {
      ...mockDashboardSnapshot.tunnel,
      kind: "running" as const,
      session_id: "session-a",
      public_url: "https://session-a.trycloudflare.com",
    },
  }

  expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, tunnelOnlySnapshot)).toBe(true)
})

test("config form emits tunnel password keep replace and clear update shape", () => {
  const form = formFromConfig(mockConfigView().config)

  const keepUpdate = updateFromForm(form)
  expect(keepUpdate.tunnel).toEqual({
    provider: "cloudflare_quick",
    auto_start: false,
    config_password_replacement: null,
    clear_config_password: false,
  })

  const replaceUpdate = updateFromForm({
    ...form,
    tunnel_password_replacement_input: "replacement tunnel password",
  })
  expect(replaceUpdate.tunnel.config_password_replacement).toBe("replacement tunnel password")
  expect(replaceUpdate.tunnel.clear_config_password).toBe(false)

  const clearUpdate = updateFromForm({
    ...form,
    tunnel: { ...form.tunnel, clear_config_password: true },
  })
  expect(clearUpdate.tunnel.config_password_replacement).toBeNull()
  expect(clearUpdate.tunnel.clear_config_password).toBe(true)
})

test("adapter schema rejects snapshots missing afk checklist", () => {
  const { afk_checklist: checklist, ...snapshotWithoutChecklist } = mockDashboardSnapshot

  expect(checklist.rows).toHaveLength(3)
  expect(() => parseAppSnapshot(snapshotWithoutChecklist)).toThrow(/Invalid input/)
})

test("adapter schema rejects malformed tunnel status payloads", () => {
  const malformedStatus = {
    ...mockTunnelStatusView(),
    kind: "ready",
  }

  expect(() => parseTunnelStatusView(malformedStatus)).toThrow(/Invalid option/)
})

test("web adapter loadSnapshot reads only the snapshot endpoint", async () => {
  const originalFetch = globalThis.fetch
  const requestedPaths: string[] = []

  Object.defineProperty(globalThis, "fetch", {
    configurable: true,
    value: async (input: Parameters<typeof fetch>[0], init: Parameters<typeof fetch>[1]) => {
      const request = new Request(input, init)
      const requestedPath = new URL(request.url).pathname
      requestedPaths.push(requestedPath)
      if (requestedPath === "/api/snapshot") {
        return new Response(JSON.stringify(mockDashboardSnapshot), {
          headers: { "content-type": "application/json" },
        })
      }
      return new Response("unexpected endpoint", { status: 500 })
    },
  })

  try {
    const adapter = createWebDashboardAdapter({ baseUrl: "http://127.0.0.1:8765" })
    await expect(adapter.loadSnapshot()).resolves.toEqual(mockDashboardSnapshot)
  } finally {
    Object.defineProperty(globalThis, "fetch", {
      configurable: true,
      value: originalFetch,
    })
  }

  expect(requestedPaths).toEqual(["/api/snapshot"])
})

function requireFakeWebSocket(): FakeWebSocket {
  if (FakeWebSocket.instance === null) {
    throw new Error("Web adapter did not construct a WebSocket")
  }
  return FakeWebSocket.instance
}

function subscribeForTest(adapter: DashboardAdapter, events: DashboardAdapterEvent[]): () => void {
  if (adapter.subscribe === undefined) {
    throw new Error(`${adapter.label} does not expose a subscription`)
  }
  return adapter.subscribe((event) => events.push(event))
}

function requireMockEvent(id: string) {
  const event = mockDashboardSnapshot.event_feed.find((item) => item.id === id)
  if (event === undefined) {
    throw new Error(`Mock event ${id} is missing`)
  }
  return event
}
