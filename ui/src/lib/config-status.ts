import type { LogLevelKey } from "@/adapters/config"
import type { ConfigFormState } from "@/components/dashboard/config-form-model"

export type RoutingGroup = {
  readonly name: string
  readonly keys: readonly LogLevelKey[]
}

export const routingGroups: readonly RoutingGroup[] = [
  { name: "Scans", keys: ["scan_incoming", "scan_easy", "scan_hard", "security_scan"] },
  {
    name: "Combat",
    keys: ["kill_easy", "kill_hard", "security_attack", "kill_rate", "no_kills", "bait_value_low"],
  },
  {
    name: "Ship & fighter",
    keys: ["ship_shields", "ship_hull", "fighter_hull", "fighter_down", "cargo_lost", "died"],
  },
  { name: "Fuel", keys: ["fuel_report", "fuel_low", "fuel_critical"] },
  { name: "Missions & rank", keys: ["missions", "missions_all", "merits", "rank_promotion"] },
  {
    name: "Session summary",
    keys: [
      "summary_kills",
      "summary_faction",
      "summary_scans",
      "summary_bounties",
      "summary_merits",
    ],
  },
  { name: "System", keys: ["duplicate_suppression"] },
]

export function routingSummary(levels: Readonly<Record<LogLevelKey, number>>): {
  readonly total: number
  readonly mention: number
  readonly off: number
} {
  const values = Object.values(levels)
  return {
    total: values.length,
    mention: values.filter((value) => value >= 2).length,
    off: values.filter((value) => value <= 0).length,
  }
}

export const configSectionIds = [
  "journal",
  "matrix",
  "tunnel",
  "web",
  "monitor",
  "routing",
] as const

export type ConfigSectionId = (typeof configSectionIds)[number]

export type SectionStatus = {
  readonly text: string
  readonly on: boolean
}

export function configSectionStatus(
  form: ConfigFormState,
): Readonly<Record<ConfigSectionId, SectionStatus>> {
  const routing = routingSummary(form.log_levels)
  const folder = form.journal.folder.trim() === "" ? "Default folder" : "Custom folder"
  return {
    journal: { text: `${folder} · ${form.journal.recent_files} recent files`, on: true },
    matrix: { text: form.matrix.enabled ? "Enabled" : "Disabled", on: form.matrix.enabled },
    tunnel: {
      text: `${tunnelProviderShortName(form.tunnel.provider)} · ${
        form.tunnel.auto_start ? "auto start" : "manual start"
      }`,
      on: true,
    },
    web: {
      text: `${form.web.host}:${form.web.port} · ${form.web.enabled ? "enabled" : "disabled"}`,
      on: form.web.enabled,
    },
    monitor: {
      text: `Warn ${form.monitor.warn_kill_rate}/h · no kills ${form.monitor.warn_no_kills_minutes} min`,
      on: true,
    },
    routing: {
      text: `${routing.total} events · ${routing.mention} mention · ${routing.off} off`,
      on: true,
    },
  }
}

function tunnelProviderShortName(provider: string): string {
  return provider === "cloudflare_quick" ? "Cloudflare" : provider
}
