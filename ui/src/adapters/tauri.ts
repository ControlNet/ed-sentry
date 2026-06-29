import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { z } from "zod"
import {
  type AppSnapshot,
  appSnapshotSchema,
  type ConfigApiView,
  type DashboardAdapter,
  DashboardAdapterError,
  type DashboardAdapterEvent,
  type EditableConfigUpdate,
  eventFeedItemSchema,
  formatAdapterError,
  parseAppSnapshot,
  parseConfigApiView,
  parseTunnelStatusView,
  type TunnelStatusView,
} from "@/adapters/types"

export type TauriDashboardTransport = {
  readonly loadSnapshot: () => Promise<unknown>
  readonly loadConfig: () => Promise<unknown>
  readonly saveConfig: (update: EditableConfigUpdate) => Promise<unknown>
  readonly loadTunnelStatus?: () => Promise<unknown>
  readonly startTunnel?: () => Promise<unknown>
  readonly listenSnapshot?: (onSnapshot: (payload: unknown) => void) => () => void
  readonly listenDashboard?: (onEvent: (payload: unknown) => void) => () => void
}

const tauriLiveUpdateSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("snapshot"),
    snapshot: appSnapshotSchema,
  }),
  z.object({
    type: z.literal("event"),
    item: eventFeedItemSchema,
  }),
])

export function createDefaultTauriDashboardAdapter(): DashboardAdapter {
  return createTauriDashboardAdapter({
    loadSnapshot: () => invoke("load_snapshot"),
    loadConfig: () => invoke("load_config"),
    saveConfig: (update) => invoke("save_config", { update }),
    loadTunnelStatus: () => invoke("load_tunnel_status"),
    startTunnel: () => invoke("start_tunnel"),
    listenDashboard(onEvent) {
      let active = true
      let unlisten: (() => void) | null = null
      void listen<unknown>("ed-sentry://dashboard", (event) => {
        if (active) {
          onEvent(event.payload)
        }
      }).then((nextUnlisten) => {
        if (active) {
          unlisten = nextUnlisten
          return
        }
        nextUnlisten()
      })
      return () => {
        active = false
        unlisten?.()
      }
    },
  })
}

export function createTauriDashboardAdapter(transport: TauriDashboardTransport): DashboardAdapter {
  const loadTunnelStatus = transport.loadTunnelStatus
  const startTunnel = transport.startTunnel
  const adapter: DashboardAdapter = {
    mode: "tauri",
    label: "Desktop service",
    async loadSnapshot(): Promise<AppSnapshot> {
      try {
        return parseAppSnapshot(await transport.loadSnapshot())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("tauri", error)
        }
        throw formatTauriCommandError(error, "Desktop adapter failed with a non-Error value")
      }
    },
    async loadConfig(): Promise<ConfigApiView> {
      try {
        return parseConfigApiView(await transport.loadConfig())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("tauri", error)
        }
        throw formatTauriCommandError(error, "Desktop config load failed with a non-Error value")
      }
    },
    async saveConfig(update: EditableConfigUpdate): Promise<ConfigApiView> {
      try {
        return parseConfigApiView(await transport.saveConfig(update))
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("tauri", error)
        }
        throw formatTauriCommandError(error, "Desktop config save failed with a non-Error value")
      }
    },
    subscribe(onEvent) {
      onEvent({
        type: "connection",
        connection: {
          status: "connected",
          label: "Desktop event bridge",
          detail: "Waiting for app-service events from the native shell",
          checkedAtDisplay: null,
        },
      })

      if (transport.listenDashboard !== undefined) {
        return transport.listenDashboard((payload) => {
          for (const event of eventsFromTauriUpdate(payload)) {
            onEvent(event)
          }
        })
      }

      if (transport.listenSnapshot === undefined) {
        return () => undefined
      }

      return transport.listenSnapshot((payload) => {
        try {
          onEvent({ type: "snapshot", snapshot: parseAppSnapshot(payload) })
        } catch (error) {
          const adapterError =
            error instanceof Error
              ? formatAdapterError("tauri", error)
              : new DashboardAdapterError(
                  "tauri",
                  "Desktop payload parser failed with a non-Error value",
                )
          onEvent({
            type: "connection",
            connection: {
              status: "degraded",
              label: "Desktop payload ignored",
              detail: adapterError.message,
              checkedAtDisplay: null,
            },
          })
        }
      })
    },
  }

  return {
    ...adapter,
    ...(loadTunnelStatus === undefined
      ? {}
      : {
          async loadTunnelStatus(): Promise<TunnelStatusView> {
            try {
              return parseTunnelStatusView(await loadTunnelStatus())
            } catch (error) {
              if (error instanceof Error) {
                throw formatAdapterError("tauri", error)
              }
              throw formatTauriCommandError(
                error,
                "Desktop tunnel status failed with a non-Error value",
              )
            }
          },
        }),
    ...(startTunnel === undefined
      ? {}
      : {
          async startTunnel(): Promise<TunnelStatusView> {
            try {
              return parseTunnelStatusView(await startTunnel())
            } catch (error) {
              if (error instanceof Error) {
                throw formatAdapterError("tauri", error)
              }
              throw formatTauriCommandError(
                error,
                "Desktop tunnel start failed with a non-Error value",
              )
            }
          },
        }),
  }
}

function eventsFromTauriUpdate(payload: unknown): readonly DashboardAdapterEvent[] {
  const parsed = tauriLiveUpdateSchema.safeParse(payload)
  if (!parsed.success) {
    return [
      {
        type: "connection",
        connection: {
          status: "degraded",
          label: "Desktop payload ignored",
          detail: "The desktop event payload did not match the dashboard protocol",
          checkedAtDisplay: null,
        },
      },
    ]
  }

  switch (parsed.data.type) {
    case "snapshot":
      return [{ type: "snapshot", snapshot: parsed.data.snapshot }]
    case "event":
      return [{ type: "event", item: parsed.data.item }]
    default:
      return assertNever(parsed.data)
  }
}

function formatTauriCommandError(error: unknown, fallbackMessage: string): DashboardAdapterError {
  if (typeof error !== "string") {
    return new DashboardAdapterError("tauri", fallbackMessage)
  }
  const message = redactSensitiveCommandText(error)
  if (message.trim() === "") {
    return new DashboardAdapterError("tauri", fallbackMessage)
  }
  return new DashboardAdapterError("tauri", message)
}

function redactSensitiveCommandText(message: string): string {
  return message
    .replaceAll(/\bBearer\s+[A-Za-z0-9._~+/=-]+/gi, "Bearer [redacted token]")
    .replaceAll(
      /\b(access[_-]?token|token|secret|password|authorization)\b\s*[:=]\s*["']?[^"',\s}]+["']?/gi,
      "$1 = [redacted]",
    )
    .replaceAll(/(?:\/home|\/Users|\/root)\/[^\s"',)]+/g, "[redacted path]")
    .replaceAll(/[A-Za-z]:\\Users\\[^\\\s"',)]+(?:\\[^\s"',)]+)*/g, "[redacted path]")
}

function assertNever(value: never): never {
  throw new Error(`Unhandled desktop event: ${String(value)}`)
}
