import { create } from "zustand"
import {
  type AppSnapshot,
  createDashboardAdapter,
  type DashboardAdapter,
  type DashboardAdapterEvent,
  type DashboardAdapterUnsubscribe,
  type DashboardConnectionState,
  type TunnelLoginResult,
  type TunnelStatusView,
} from "@/adapters/dashboard"
import {
  normalizeEventFeed,
  normalizeSnapshot,
  shouldApplySnapshotUpdate,
} from "./snapshot-normalization"

export type DashboardStatus = "idle" | "loading" | "ready" | "error"

type DashboardState = {
  readonly adapter: DashboardAdapter
  readonly snapshot: AppSnapshot | null
  readonly status: DashboardStatus
  readonly connection: DashboardConnectionState
  readonly errorMessage: string | null
  readonly unsubscribe: DashboardAdapterUnsubscribe | null
  readonly start: () => Promise<void>
  readonly refresh: () => Promise<void>
  readonly refreshTunnelStatus: () => Promise<void>
  readonly startTunnel: () => Promise<void>
  readonly loginTunnel: (password: string) => Promise<TunnelLoginResult>
}

const liveSnapshotRefreshMs = 1_000

const initialConnection: DashboardConnectionState = {
  status: "idle",
  label: "Adapter idle",
  detail: "Dashboard transport has not connected yet",
  checkedAtDisplay: null,
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
  adapter: createDashboardAdapter(),
  snapshot: null,
  status: "idle",
  connection: initialConnection,
  errorMessage: null,
  unsubscribe: null,
  async start() {
    const { adapter, unsubscribe } = get()
    if (unsubscribe === null && adapter.subscribe !== undefined) {
      const adapterUnsubscribe = adapter.subscribe((event) => applyAdapterEvent(event, set, get))
      const refreshUnsubscribe = startLiveSnapshotRefresh(adapter, set, get)
      const nextUnsubscribe = () => {
        adapterUnsubscribe()
        refreshUnsubscribe()
      }
      set({ unsubscribe: nextUnsubscribe })
    }
    await get().refresh()
  },
  async refresh() {
    const adapter = get().adapter
    set({
      status: "loading",
      errorMessage: null,
      connection: {
        status: "loading",
        label: adapter.label,
        detail: "Loading dashboard snapshot",
        checkedAtDisplay: null,
      },
    })
    try {
      const snapshot = await loadNormalizedSnapshot(adapter)
      setSnapshotFromRefresh(snapshot, adapter, set)
    } catch (error) {
      if (error instanceof Error) {
        set({
          status: "error",
          errorMessage: error.message,
          connection: {
            status: "error",
            label: adapter.label,
            detail: error.message,
            checkedAtDisplay: null,
          },
        })
        return
      }
      throw error
    }
  },
  async refreshTunnelStatus() {
    const loadTunnelStatus = get().adapter.loadTunnelStatus
    if (loadTunnelStatus === undefined) {
      return
    }
    setSnapshotTunnel(await loadTunnelStatus(), set, get)
  },
  async startTunnel() {
    const startTunnel = get().adapter.startTunnel
    if (startTunnel === undefined) {
      return
    }
    setSnapshotTunnel(await startTunnel(), set, get)
  },
  async loginTunnel(password: string) {
    const loginTunnel = get().adapter.loginTunnel
    if (loginTunnel === undefined) {
      return {
        ok: false,
        code: "tunnel_login_unsupported",
        message: "Tunnel login is not available in this adapter",
      }
    }
    const result = await loginTunnel(password)
    if (result.ok) {
      await get().refreshTunnelStatus()
    }
    return result
  },
}))

function startLiveSnapshotRefresh(
  adapter: DashboardAdapter,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): DashboardAdapterUnsubscribe {
  let refreshInFlight = false
  const refresh = () => {
    if (refreshInFlight) {
      return
    }
    refreshInFlight = true
    void loadNormalizedSnapshot(adapter)
      .then((snapshot) => setNormalizedSnapshot(snapshot, set, get))
      .catch((error: unknown) => {
        if (error instanceof Error) {
          set({
            connection: {
              status: "degraded",
              label: adapter.label,
              detail: error.message,
              checkedAtDisplay: get().snapshot?.generated_at_display ?? null,
            },
          })
          return
        }
        throw error
      })
      .finally(() => {
        refreshInFlight = false
      })
  }
  const interval = globalThis.setInterval(refresh, liveSnapshotRefreshMs)
  return () => globalThis.clearInterval(interval)
}

async function loadNormalizedSnapshot(adapter: DashboardAdapter): Promise<AppSnapshot> {
  return normalizeSnapshot(await adapter.loadSnapshot())
}

function setSnapshotFromRefresh(
  snapshot: AppSnapshot,
  adapter: DashboardAdapter,
  set: (state: Partial<DashboardState>) => void,
): void {
  set({
    snapshot,
    status: "ready",
    connection: {
      status: "connected",
      label: adapter.label,
      detail: snapshot.web.message ?? snapshot.web.status_label,
      checkedAtDisplay: snapshot.generated_at_display,
    },
  })
}

function applyAdapterEvent(
  event: DashboardAdapterEvent,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): void {
  switch (event.type) {
    case "connection":
      set({ connection: event.connection })
      return
    case "snapshot":
      setNormalizedSnapshot(event.snapshot, set, get)
      return
    case "event": {
      const currentSnapshot = get().snapshot
      if (currentSnapshot === null) {
        return
      }
      set({
        snapshot: {
          ...currentSnapshot,
          event_feed: mergeEventFeed(event.item, currentSnapshot.event_feed),
        },
      })
      return
    }
    default:
      assertNever(event)
  }
}

function setNormalizedSnapshot(
  snapshot: AppSnapshot,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): void {
  const currentSnapshot = get().snapshot
  const normalizedSnapshot = normalizeSnapshot(snapshot)
  if (currentSnapshot !== null && !shouldApplySnapshotUpdate(currentSnapshot, normalizedSnapshot)) {
    set({ status: "ready" })
    return
  }
  set({ snapshot: normalizedSnapshot, status: "ready" })
}

function setSnapshotTunnel(
  tunnel: TunnelStatusView,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): void {
  const currentSnapshot = get().snapshot
  if (currentSnapshot === null) {
    return
  }
  set({
    snapshot: {
      ...currentSnapshot,
      tunnel,
    },
  })
}

function mergeEventFeed(
  nextItem: AppSnapshot["event_feed"][number],
  currentItems: AppSnapshot["event_feed"],
): AppSnapshot["event_feed"] {
  return normalizeEventFeed([
    nextItem,
    ...currentItems.filter((currentItem) => currentItem.id !== nextItem.id),
  ])
}

function assertNever(value: never): never {
  throw new Error(`Unhandled dashboard adapter event: ${String(value)}`)
}
