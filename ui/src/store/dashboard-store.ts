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
  readonly retry: () => Promise<void>
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

export function createDashboardStore(adapterOverride?: DashboardAdapter) {
  return create<DashboardState>((set, get) => ({
    adapter: adapterOverride ?? createDashboardAdapter(),
    snapshot: null,
    status: "idle",
    connection: initialConnection,
    errorMessage: null,
    unsubscribe: null,
    async start() {
      const { adapter, unsubscribe } = get()
      if (unsubscribe === null) {
        const adapterUnsubscribe = adapter.subscribe?.((event) =>
          applyAdapterEvent(event, set, get),
        )
        const refreshUnsubscribe = startLiveSnapshotRefresh(adapter, set, get)
        const nextUnsubscribe = () => {
          adapterUnsubscribe?.()
          refreshUnsubscribe()
        }
        set({ unsubscribe: nextUnsubscribe })
      }
      await get().refresh()
    },
    async refresh() {
      await loadIntoStore(get().adapter, "load", set)
    },
    async retry() {
      await loadIntoStore(get().adapter, "retry", set)
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
}

export const useDashboardStore = createDashboardStore()

async function loadIntoStore(
  adapter: DashboardAdapter,
  mode: "load" | "retry",
  set: (state: Partial<DashboardState>) => void,
): Promise<void> {
  set({
    status: "loading",
    errorMessage: null,
    connection: {
      status: "loading",
      label: adapter.label,
      detail: mode === "retry" ? "Retrying dashboard startup" : "Loading dashboard snapshot",
      checkedAtDisplay: null,
    },
  })
  try {
    const raw =
      mode === "retry" && adapter.retryStartup !== undefined
        ? await adapter.retryStartup()
        : await adapter.loadSnapshot()
    const snapshot = normalizeSnapshot(raw)
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
}

function startLiveSnapshotRefresh(
  adapter: DashboardAdapter,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): DashboardAdapterUnsubscribe {
  let active = true
  let refreshInFlight = false
  const refresh = () => {
    const state = get()
    const shouldRecoverWebStartup =
      adapter.mode === "web" && state.snapshot === null && state.status === "error"
    if (!active || refreshInFlight || state.status === "loading") {
      return
    }
    if (state.snapshot === null && !shouldRecoverWebStartup) {
      return
    }
    refreshInFlight = true
    void loadNormalizedSnapshot(adapter)
      .then((snapshot) => {
        if (!active) {
          return
        }
        if (get().snapshot === null) {
          setSnapshotFromRefresh(snapshot, adapter, set)
          return
        }
        setNormalizedSnapshot(snapshot, adapter, set, get)
      })
      .catch((error: unknown) => {
        if (!active) {
          return
        }
        if (error instanceof Error) {
          const currentSnapshot = get().snapshot
          if (currentSnapshot === null) {
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
          set({
            connection: {
              status: "degraded",
              label: adapter.label,
              detail: error.message,
              checkedAtDisplay: currentSnapshot.generated_at_display,
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
  return () => {
    active = false
    globalThis.clearInterval(interval)
  }
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
    errorMessage: null,
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
      setNormalizedSnapshot(event.snapshot, get().adapter, set, get)
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
  adapter: DashboardAdapter,
  set: (state: Partial<DashboardState>) => void,
  get: () => DashboardState,
): void {
  const state = get()
  const currentSnapshot = state.snapshot
  const normalizedSnapshot = normalizeSnapshot(snapshot)
  const recoveredConnection =
    state.connection.status === "error" || state.connection.status === "degraded"
      ? {
          connection: {
            status: "connected" as const,
            label: adapter.label,
            detail: normalizedSnapshot.web.message ?? normalizedSnapshot.web.status_label,
            checkedAtDisplay: normalizedSnapshot.generated_at_display,
          },
        }
      : {}
  if (currentSnapshot !== null && !shouldApplySnapshotUpdate(currentSnapshot, normalizedSnapshot)) {
    set({ status: "ready", errorMessage: null, ...recoveredConnection })
    return
  }
  set({ snapshot: normalizedSnapshot, status: "ready", errorMessage: null, ...recoveredConnection })
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
