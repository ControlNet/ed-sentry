import { z } from "zod"

export const logLevelKeys = [
  "scan_incoming",
  "scan_easy",
  "scan_hard",
  "kill_easy",
  "kill_hard",
  "fighter_hull",
  "fighter_down",
  "ship_shields",
  "ship_hull",
  "died",
  "cargo_lost",
  "bait_value_low",
  "security_scan",
  "security_attack",
  "fuel_report",
  "fuel_low",
  "fuel_critical",
  "missions",
  "missions_all",
  "merits",
  "rank_promotion",
  "no_kills",
  "kill_rate",
  "summary_kills",
  "summary_faction",
  "summary_scans",
  "summary_bounties",
  "summary_merits",
  "duplicate_suppression",
] as const

const logLevelConfigSchema = z
  .object({
    scan_incoming: z.number(),
    scan_easy: z.number(),
    scan_hard: z.number(),
    kill_easy: z.number(),
    kill_hard: z.number(),
    fighter_hull: z.number(),
    fighter_down: z.number(),
    ship_shields: z.number(),
    ship_hull: z.number(),
    died: z.number(),
    cargo_lost: z.number(),
    bait_value_low: z.number(),
    security_scan: z.number(),
    security_attack: z.number(),
    fuel_report: z.number(),
    fuel_low: z.number(),
    fuel_critical: z.number(),
    missions: z.number(),
    missions_all: z.number(),
    merits: z.number(),
    rank_promotion: z.number(),
    no_kills: z.number(),
    kill_rate: z.number(),
    summary_kills: z.number(),
    summary_faction: z.number(),
    summary_scans: z.number(),
    summary_bounties: z.number(),
    summary_merits: z.number(),
    duplicate_suppression: z.number(),
  })
  .readonly()

export const configApiViewSchema = z.object({
  version: z.number(),
  config: z.object({
    journal: z.object({
      folder: z.string(),
      recent_files: z.number(),
    }),
    monitor: z.object({
      use_utc: z.boolean(),
      live_status: z.boolean(),
      dynamic_title: z.boolean(),
      warn_kill_rate: z.number(),
      warn_kill_rate_delay_minutes: z.number(),
      warn_no_kills_minutes: z.number(),
      warn_no_kills_initial_minutes: z.number(),
      warn_cooldown_minutes: z.number(),
      duplicate_max: z.number(),
      pirate_names: z.boolean(),
      bounty_faction: z.boolean(),
      bounty_value: z.boolean(),
      extended_stats: z.boolean(),
      min_scan_level: z.number(),
      poll_interval_ms: z.number(),
    }),
    log_levels: logLevelConfigSchema,
    matrix: z
      .object({
        enabled: z.boolean(),
        homeserver: z.string().nullable().optional(),
        user_id: z.string().nullable().optional(),
        room_id: z.string().nullable().optional(),
        mention_user_id: z.string().nullable().optional(),
        status_update_interval_seconds: z.number(),
        access_token_present: z.boolean(),
      })
      .nullable()
      .optional(),
    web: z.object({
      enabled: z.boolean(),
      host: z.string(),
      port: z.number(),
      open_browser: z.boolean(),
      status_label: z.string(),
    }),
  }),
  policy: z.object({
    state_changing_enabled: z.boolean(),
    state_changing_reason: z.string(),
    remote_bind: z.boolean(),
  }),
})

export type LogLevelKey = (typeof logLevelKeys)[number]
export type ConfigApiView = Readonly<z.infer<typeof configApiViewSchema>>
export type EditableConfigView = ConfigApiView["config"]
export type MatrixConfigView = NonNullable<EditableConfigView["matrix"]>

export type EditableConfigUpdate = {
  readonly journal: {
    readonly folder: string
    readonly recent_files: number
  }
  readonly monitor: EditableConfigView["monitor"]
  readonly log_levels: Record<LogLevelKey, number>
  readonly matrix: {
    readonly enabled: boolean
    readonly homeserver: string | null
    readonly user_id: string | null
    readonly room_id: string | null
    readonly mention_user_id: string | null
    readonly status_update_interval_seconds: number
    readonly access_token_replacement: string | null
    readonly clear_access_token: boolean
  }
  readonly web: {
    readonly enabled: boolean
    readonly host: string
    readonly port: number
    readonly open_browser: boolean
  }
}

export function parseConfigApiView(payload: unknown): ConfigApiView {
  return configApiViewSchema.parse(payload)
}
