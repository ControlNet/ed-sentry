import { z } from "zod"
import {
  appSnapshotSchema,
  type DashboardAdapterEvent,
  type DashboardAdapterUnsubscribe,
  eventFeedItemSchema,
} from "./types"

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

type ParsedWebSocketJson =
  | {
      readonly ok: true
      readonly payload: unknown
    }
  | {
      readonly ok: false
      readonly message: string
    }

export function createWebSocketSubscription(
  baseUrl: string,
  eventsPath: string,
  onEvent: (event: DashboardAdapterEvent) => void,
): DashboardAdapterUnsubscribe {
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
      return [{ type: "snapshot", snapshot: message.snapshot }]
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
