import { z } from "zod"
import type { ConfigApiView, EditableConfigUpdate } from "@/adapters/config"
import { configApiViewSchema, parseConfigApiView } from "@/adapters/config"
import { tunnelStatusViewSchema } from "@/adapters/tunnel"
import type {
  TunnelLoginResult,
  TunnelProvider,
  TunnelStatusKind,
  TunnelStatusView,
} from "./tunnel"

export const adapterModes = ["mock", "web", "tauri"] as const
export const serviceStatusKinds = ["disabled", "starting", "running", "warning", "error"] as const
export const missionStates = ["active", "redirected", "completed", "failed", "abandoned"] as const
export const missionKinds = ["massacre", "trade", "other"] as const
export const connectionStatuses = ["idle", "loading", "connected", "degraded", "error"] as const
export const afkChecklistStates = ["pass", "fail", "unknown"] as const
export const afkChecklistSources = ["Status.json", "Cargo.json", "unknown"] as const

export type AdapterMode = (typeof adapterModes)[number]
export type ServiceStatusKind = (typeof serviceStatusKinds)[number]
export type MissionState = (typeof missionStates)[number]
export type MissionKind = (typeof missionKinds)[number]
export type ConnectionStatus = (typeof connectionStatuses)[number]
export type AfkChecklistState = (typeof afkChecklistStates)[number]
export type AfkChecklistSource = (typeof afkChecklistSources)[number]

const valueDisplayNumberSchema = z.object({
  value: z.number(),
  display: z.string(),
})

const rateViewSchema = z.object({
  value: z.number(),
  display: z.string(),
})

export const sessionViewSchema = z.object({
  commander: z.string().nullable().optional(),
  ship: z.string().nullable().optional(),
  system: z.string().nullable().optional(),
  mode: z.string().nullable().optional(),
  active: z.boolean(),
  status_label: z.string(),
  started_at: z.string().nullable().optional(),
  started_at_display: z.string().nullable().optional(),
  ended_at: z.string().nullable().optional(),
  ended_at_display: z.string().nullable().optional(),
  elapsed_seconds: z.number(),
  elapsed_display: z.string(),
  shields_up: z.boolean().nullable().optional(),
  shields_display: z.string(),
  ship_hull_percent: z.number().nullable().optional(),
  ship_hull_display: z.string(),
  fighter_hull_percent: z.number().nullable().optional(),
  fighter_hull_display: z.string(),
  fighter_alive: z.boolean().nullable().optional(),
  kills: z.number(),
  scans: z.number(),
  bounty_total: valueDisplayNumberSchema,
  merits: z.number(),
  merits_to_report: z.number(),
  kill_total_rate_per_hour: rateViewSchema,
  kill_recent_rate_per_hour: rateViewSchema,
  scan_total_rate_per_hour: rateViewSchema,
  scan_recent_rate_per_hour: rateViewSchema,
  last_kill_at: z.string().nullable().optional(),
  last_kill_display: z.string().nullable().optional(),
  last_scan_at: z.string().nullable().optional(),
  last_scan_display: z.string().nullable().optional(),
})

const missionProgressSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("none") }),
  z.object({
    kind: z.literal("massacre"),
    target: z.string().nullable().optional(),
    target_faction: z.string().nullable().optional(),
    kills: z.number(),
    kill_count: z.number(),
    display: z.string(),
  }),
  z.object({
    kind: z.literal("trade"),
    commodity: z.string().nullable().optional(),
    collected: z.number(),
    delivered: z.number(),
    count: z.number(),
    display: z.string(),
  }),
])

export const missionViewSchema = z.object({
  mission_id: z.number(),
  state: z.enum(missionStates),
  state_label: z.string(),
  kind: z.enum(missionKinds),
  kind_label: z.string(),
  display_name: z.string(),
  issuing_faction: z.string().nullable().optional(),
  target_faction: z.string().nullable().optional(),
  destination_system: z.string().nullable().optional(),
  destination_station: z.string().nullable().optional(),
  accepted_at: z.string(),
  accepted_at_display: z.string(),
  expiry: z.string().nullable().optional(),
  expiry_display: z.string().nullable().optional(),
  reward: valueDisplayNumberSchema,
  progress: missionProgressSchema,
})

export const missionListViewSchema = z.object({
  active_count: z.number(),
  completed_count: z.number(),
  total_count: z.number(),
  status_label: z.string(),
  items: z.array(missionViewSchema).default([]),
})

export const notificationViewSchema = z.object({
  event_type: z.string(),
  level: z.number(),
  alert_level: z.string(),
  emoji: z.string().nullable().optional(),
  text: z.string(),
  timestamp: z.string(),
  timestamp_display: z.string(),
  mention: z.boolean(),
})

export const eventFeedItemSchema = z.object({
  id: z.string(),
  source: z.string(),
  event_type: z.string(),
  level: z.number(),
  summary: z.string(),
  timestamp: z.string(),
  timestamp_display: z.string(),
})

export const journalSourceViewSchema = z.object({
  folder: z.string(),
  selected_file: z.string().nullable().optional(),
  recent_files: z.number(),
  status_label: z.string(),
})

export const serviceStatusViewSchema = z.object({
  kind: z.enum(serviceStatusKinds),
  status_label: z.string(),
  message: z.string().nullable().optional(),
  room_id: z.string().nullable().optional(),
  bind_address: z.string().nullable().optional(),
  url: z.string().nullable().optional(),
  checked_at: z.string().nullable().optional(),
  checked_at_display: z.string().nullable().optional(),
})

export const afkChecklistRowViewSchema = z.object({
  id: z.string(),
  label: z.string(),
  detail: z.string(),
  state: z.enum(afkChecklistStates),
  source: z.enum(afkChecklistSources),
})

export const afkChecklistViewSchema = z.object({
  rows: z.array(afkChecklistRowViewSchema),
})

export const appSnapshotSchema = z.object({
  generated_at: z.string(),
  generated_at_display: z.string(),
  session: sessionViewSchema,
  missions: missionListViewSchema,
  afk_checklist: afkChecklistViewSchema,
  notifications: z.array(notificationViewSchema).default([]),
  event_feed: z.array(eventFeedItemSchema).default([]),
  journal_source: journalSourceViewSchema,
  matrix: serviceStatusViewSchema,
  web: serviceStatusViewSchema,
  tunnel: tunnelStatusViewSchema,
})

export type ValueDisplayNumber = Readonly<z.infer<typeof valueDisplayNumberSchema>>
export type RateView = Readonly<z.infer<typeof rateViewSchema>>
export type SessionView = Readonly<z.infer<typeof sessionViewSchema>>
export type MissionProgressView = Readonly<z.infer<typeof missionProgressSchema>>
export type MissionView = Readonly<z.infer<typeof missionViewSchema>>
export type MissionListView = Readonly<z.infer<typeof missionListViewSchema>>
export type NotificationView = Readonly<z.infer<typeof notificationViewSchema>>
export type EventFeedItem = Readonly<z.infer<typeof eventFeedItemSchema>>
export type JournalSourceView = Readonly<z.infer<typeof journalSourceViewSchema>>
export type ServiceStatusView = Readonly<z.infer<typeof serviceStatusViewSchema>>
export type AfkChecklistRowView = Readonly<z.infer<typeof afkChecklistRowViewSchema>>
export type AfkChecklistView = Readonly<z.infer<typeof afkChecklistViewSchema>>
export type AppSnapshot = Readonly<z.infer<typeof appSnapshotSchema>>

export type DashboardConnectionState = {
  readonly status: ConnectionStatus
  readonly label: string
  readonly detail: string
  readonly checkedAtDisplay: string | null
}

export type DashboardAdapterEvent =
  | {
      readonly type: "snapshot"
      readonly snapshot: AppSnapshot
    }
  | {
      readonly type: "event"
      readonly item: EventFeedItem
    }
  | {
      readonly type: "connection"
      readonly connection: DashboardConnectionState
    }

export type DashboardAdapterUnsubscribe = () => void

export interface DashboardAdapter {
  readonly mode: AdapterMode
  readonly label: string
  loadSnapshot(): Promise<AppSnapshot>
  loadConfig(): Promise<ConfigApiView>
  saveConfig(update: EditableConfigUpdate): Promise<ConfigApiView>
  loadTunnelStatus?(): Promise<TunnelStatusView>
  startTunnel?(): Promise<TunnelStatusView>
  loginTunnel?(password: string): Promise<TunnelLoginResult>
  subscribe?(onEvent: (event: DashboardAdapterEvent) => void): DashboardAdapterUnsubscribe
}

export class DashboardAdapterError extends Error {
  readonly adapterMode: AdapterMode

  constructor(adapterMode: AdapterMode, message: string) {
    super(message)
    this.name = "DashboardAdapterError"
    this.adapterMode = adapterMode
  }
}

export function parseAppSnapshot(payload: unknown): AppSnapshot {
  return appSnapshotSchema.parse(payload)
}

export {
  parseTunnelStatusView,
  tunnelProviders,
  tunnelStatusKinds,
  tunnelStatusViewSchema,
} from "./tunnel"
export type {
  ConfigApiView,
  EditableConfigUpdate,
  TunnelLoginResult,
  TunnelProvider,
  TunnelStatusKind,
  TunnelStatusView,
}
export { configApiViewSchema, parseConfigApiView }

export function formatAdapterError(
  adapterMode: AdapterMode,
  error: unknown,
): DashboardAdapterError {
  if (error instanceof DashboardAdapterError) {
    return error
  }
  if (error instanceof Error) {
    return new DashboardAdapterError(adapterMode, error.message)
  }
  return new DashboardAdapterError(adapterMode, "Adapter returned an unknown error")
}
