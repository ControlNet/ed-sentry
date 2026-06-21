import ky from "ky"
import { z } from "zod"
import type { ConfigApiView, EditableConfigUpdate } from "@/adapters/config"
import {
  type AppSnapshot,
  appSnapshotSchema,
  type DashboardAdapter,
  DashboardAdapterError,
  type DashboardAdapterEvent,
  eventFeedItemSchema,
  formatAdapterError,
  parseAppSnapshot,
  parseConfigApiView,
} from "@/adapters/types"

const webSocketMessageSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("hello"),
    snapshot: appSnapshotSchema,
    event_feed: z.array(eventFeedItemSchema).default([]),
  }),
  z.object({
    type: z.literal("snapshot"),
    snapshot: appSnapshotSchema,
  }),
  z.object({
    type: z.literal("event"),
    item: eventFeedItemSchema,
  }),
  z.object({
    type: z.literal("error"),
    error: z.object({
      code: z.string(),
      message: z.string(),
    }),
  }),
])

export type WebDashboardAdapterOptions = {
  readonly baseUrl?: string
  readonly snapshotPath?: string
  readonly configPath?: string
  readonly eventsPath?: string
  readonly timeoutMs?: number
}

export function createWebDashboardAdapter(
  options: WebDashboardAdapterOptions = {},
): DashboardAdapter {
  const baseUrl = options.baseUrl ?? globalThis.location.origin
  const snapshotPath = options.snapshotPath ?? "api/snapshot"
  const configPath = options.configPath ?? "api/config"
  const eventsPath = options.eventsPath ?? "/api/events"
  const timeout = options.timeoutMs ?? 5_000
  const api = ky.create({
    baseUrl,
    timeout,
    retry: 0,
  })

  return {
    mode: "web",
    label: "Web API",
    async loadSnapshot(): Promise<AppSnapshot> {
      try {
        const snapshotPayload = await api.get(snapshotPath).json<unknown>()
        return parseAppSnapshot(snapshotPayload)
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web adapter failed with a non-Error value")
      }
    },
    async loadConfig(): Promise<ConfigApiView> {
      try {
        return parseConfigApiView(await api.get(configPath).json<unknown>())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web config load failed with a non-Error value")
      }
    },
    async saveConfig(update: EditableConfigUpdate): Promise<ConfigApiView> {
      try {
        return parseConfigApiView(await api.put(configPath, { json: update }).json<unknown>())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web config save failed with a non-Error value")
      }
    },
    subscribe(onEvent) {
      const webSocket = new WebSocket(toWebSocketUrl(baseUrl, eventsPath))

      webSocket.addEventListener("open", () => {
        onEvent({
          type: "connection",
          connection: {
            status: "connected",
            label: "WebSocket connected",
            detail: "Receiving dashboard events from the WebUI server",
            checkedAtDisplay: null,
          },
        })
      })

      webSocket.addEventListener("message", (message) => {
        const parsedJson = parseWebSocketJson(message.data)
        if (!parsedJson.ok) {
          onEvent({
            type: "connection",
            connection: {
              status: "degraded",
              label: "Message ignored",
              detail: parsedJson.message,
              checkedAtDisplay: null,
            },
          })
          return
        }

        const parsed = webSocketMessageSchema.safeParse(parsedJson.payload)
        if (!parsed.success) {
          onEvent({
            type: "connection",
            connection: {
              status: "degraded",
              label: "Message ignored",
              detail: "The WebSocket payload did not match the dashboard protocol",
              checkedAtDisplay: null,
            },
          })
          return
        }

        for (const event of eventsFromWebSocketMessage(parsed.data)) {
          onEvent(event)
        }
      })

      webSocket.addEventListener("error", () => {
        onEvent({
          type: "connection",
          connection: {
            status: "error",
            label: "WebSocket error",
            detail: "Live updates are unavailable",
            checkedAtDisplay: null,
          },
        })
      })

      webSocket.addEventListener("close", () => {
        onEvent({
          type: "connection",
          connection: {
            status: "degraded",
            label: "WebSocket closed",
            detail: "Dashboard is showing the last received snapshot",
            checkedAtDisplay: null,
          },
        })
      })

      return () => webSocket.close()
    },
  }
}

function toWebSocketUrl(baseUrl: string, path: string): string {
  const url = new URL(path, baseUrl)
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:"
  return url.toString()
}

function eventsFromWebSocketMessage(
  message: z.infer<typeof webSocketMessageSchema>,
): readonly DashboardAdapterEvent[] {
  switch (message.type) {
    case "hello":
      return [
        { type: "snapshot", snapshot: message.snapshot },
        ...message.event_feed.map(
          (item): DashboardAdapterEvent => ({
            type: "event",
            item,
          }),
        ),
      ]
    case "snapshot":
      return [{ type: "snapshot", snapshot: message.snapshot }]
    case "event":
      return [{ type: "event", item: message.item }]
    case "error":
      return [
        {
          type: "connection",
          connection: {
            status: "error",
            label: "Server error",
            detail: message.error.message,
            checkedAtDisplay: null,
          },
        },
      ]
    default:
      return assertNever(message)
  }
}

type ParsedWebSocketJson =
  | {
      readonly ok: true
      readonly payload: unknown
    }
  | {
      readonly ok: false
      readonly message: string
    }

function parseWebSocketJson(payload: unknown): ParsedWebSocketJson {
  try {
    return { ok: true, payload: JSON.parse(String(payload)) }
  } catch (error) {
    if (error instanceof SyntaxError) {
      return {
        ok: false,
        message: "The WebSocket message was not valid JSON",
      }
    }
    throw error
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled WebSocket message: ${String(value)}`)
}
