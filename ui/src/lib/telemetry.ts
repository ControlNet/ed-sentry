import type { AfkChecklistRowView, MissionView } from "@/adapters/types"
import { formatRate } from "./format"

/** Percentage of `max` covered by `value`, clamped to 0..100. */
export function scalePercent(value: number, max: number): number {
  if (max <= 0) {
    return 0
  }
  return Math.min(100, Math.max(0, (value / max) * 100))
}

/**
 * Shared scale for rate bars: the base scale, or the next multiple of half the
 * base when a value is larger, so every bar stays inside its track.
 */
export function rateScaleMax(values: readonly number[], base: number): number {
  const largest = Math.max(0, ...values)
  if (largest <= base) {
    return base
  }
  const step = base / 2
  return Math.ceil(largest / step) * step
}

/** Hourly rate text; "-/h" before the first event, matching the CLI status line. */
export function rateDisplay(eventCount: number, perHour: number): string {
  return eventCount === 0 ? "-/h" : formatRate(perHour)
}

/** Recent rate relative to the session average, in whole percent. */
export function trendPercent(recent: number, average: number): number | null {
  if (average <= 0) {
    return null
  }
  return Math.round(((recent - average) / average) * 100)
}

export function formatTrend(percent: number): string {
  return percent >= 0 ? `▲ +${percent}%` : `▼ −${Math.abs(percent)}%`
}

export type NoKillsProgress = {
  readonly seconds: number
  /** Share of the warning threshold used, 0..1; null when the warning is disabled. */
  readonly fraction: number | null
  readonly over: boolean
}

export function noKillsProgress(
  lastKillMs: number,
  nowMs: number,
  warnMinutes: number,
): NoKillsProgress {
  const seconds = Math.max(0, Math.floor((nowMs - lastKillMs) / 1000))
  if (warnMinutes <= 0) {
    return { seconds, fraction: null, over: false }
  }
  const limit = warnMinutes * 60
  return { seconds, fraction: Math.min(1, seconds / limit), over: seconds >= limit }
}

/** Number of lit segments for a 0..1 fraction. */
export function segmentsOn(fraction: number | null | undefined, count: number): number {
  if (fraction === null || fraction === undefined) {
    return 0
  }
  return Math.round(Math.min(1, Math.max(0, fraction)) * count)
}

export type SharedKillProgress = {
  readonly completed: number
  readonly total: number
  readonly remaining: number
}

type IssuerKillGroup = {
  readonly required: number
  readonly remaining: number
}

/**
 * Kill progress of the massacre stack. Kills against one target faction count
 * for every issuer at once, so each target faction contributes the largest
 * per-issuer requirement; missions from the same issuer run one after another
 * and are summed first.
 */
export function sharedKillProgress(missions: readonly MissionView[]): SharedKillProgress {
  const stacks = new Map<string, Map<string, IssuerKillGroup>>()

  for (const mission of missions) {
    if (mission.progress.kind !== "massacre") {
      continue
    }
    const stackKey = mission.progress.target_faction ?? `mission:${mission.mission_id}`
    const issuerKey = mission.issuing_faction ?? `mission:${mission.mission_id}`
    const issuerGroups = stacks.get(stackKey) ?? new Map<string, IssuerKillGroup>()
    const current = issuerGroups.get(issuerKey) ?? { required: 0, remaining: 0 }
    const remaining = Math.max(0, mission.progress.kill_count - mission.progress.kills)
    issuerGroups.set(issuerKey, {
      required: current.required + mission.progress.kill_count,
      remaining: current.remaining + remaining,
    })
    stacks.set(stackKey, issuerGroups)
  }

  let total = 0
  let remaining = 0
  for (const issuerGroups of stacks.values()) {
    total += maxIssuerValue(issuerGroups, "required")
    remaining += maxIssuerValue(issuerGroups, "remaining")
  }
  return { completed: Math.max(0, total - remaining), total, remaining }
}

function maxIssuerValue(
  issuerGroups: ReadonlyMap<string, IssuerKillGroup>,
  field: keyof IssuerKillGroup,
): number {
  let value = 0
  for (const group of issuerGroups.values()) {
    value = Math.max(value, group[field])
  }
  return value
}

/** Minutes until the remaining kills are done at the given rate. */
export function missionEtaMinutes(remainingKills: number, killRatePerHour: number): number | null {
  if (remainingKills <= 0) {
    return 0
  }
  if (killRatePerHour <= 0) {
    return null
  }
  return Math.max(1, Math.round((remainingKills / killRatePerHour) * 60))
}

export function missionEtaLabel(remainingKills: number, killRatePerHour: number): string {
  const minutes = missionEtaMinutes(remainingKills, killRatePerHour)
  if (minutes === null) {
    return "—"
  }
  if (minutes === 0) {
    return "Complete"
  }
  if (minutes < 60) {
    return `${minutes}m`
  }
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`
}

export function readinessSummary(rows: readonly AfkChecklistRowView[]): {
  readonly pass: number
  readonly total: number
} {
  return { pass: rows.filter((row) => row.state === "pass").length, total: rows.length }
}

/**
 * Current time on the snapshot clock: the snapshot's generation time plus the
 * local time elapsed since it was received. This avoids depending on the
 * agreement between the local clock and the clock that produced the snapshot.
 */
export function liveNowMs(generatedMs: number, receivedMs: number, currentMs: number): number {
  return generatedMs + Math.max(0, currentMs - receivedMs)
}
