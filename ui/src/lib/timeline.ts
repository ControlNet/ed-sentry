import type { MissionView } from "@/adapters/types"
import { parseTimestamp, utcClock, utcHourMinute } from "./format"

const MINUTE_MS = 60_000
const HOUR_MS = 60 * MINUTE_MS
const DAY_MS = 24 * HOUR_MS
const MAX_HOUR_LABELS = 9
const MAX_SESSION_INNER_LABELS = 6
const MISSION_STEPS_MS = [1, 2, 3, 4, 6, 12, 24].map((hours) => hours * HOUR_MS)
const SESSION_STEPS_MS = [
  15 * MINUTE_MS,
  30 * MINUTE_MS,
  ...MISSION_STEPS_MS.filter((step) => step >= HOUR_MS),
]
const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
] as const

export type AxisLabel = {
  readonly label: string
  /** Position on the axis in percent. */
  readonly left: number
}

export type MissionTimelineRow = {
  readonly missionId: number
  readonly acceptedMs: number
  readonly expiryMs: number
  readonly left: number
  readonly width: number
  readonly elapsedWidth: number
  readonly remainingSeconds: number
}

export type MissionTimeline = {
  readonly start: number
  readonly end: number
  readonly hours: readonly AxisLabel[]
  readonly nowLeft: number
  readonly etaLeft: number | null
  readonly rows: readonly MissionTimelineRow[]
}

/**
 * Accepted-to-expiry tracks on one shared axis that covers whole UTC hours from
 * the earliest acceptance (or now) to the latest expiry (or ETA).
 */
export function missionTimeline(
  missions: readonly MissionView[],
  nowMs: number,
  etaMs: number | null,
): MissionTimeline {
  const spans = missions.flatMap((mission) => {
    const acceptedMs = parseTimestamp(mission.accepted_at)
    const expiryMs = parseTimestamp(mission.expiry)
    if (acceptedMs === null || expiryMs === null) {
      return []
    }
    return [{ missionId: mission.mission_id, acceptedMs, expiryMs }]
  })

  const earliest = Math.min(nowMs, ...spans.map((span) => span.acceptedMs))
  const latest = Math.max(nowMs, etaMs ?? nowMs, ...spans.map((span) => span.expiryMs))
  const start = Math.floor(earliest / HOUR_MS) * HOUR_MS
  const end = Math.max(start + HOUR_MS, Math.ceil(latest / HOUR_MS) * HOUR_MS)
  const span = end - start
  const at = (ms: number): number => Math.min(100, Math.max(0, ((ms - start) / span) * 100))

  const step = axisStep(start, end, MAX_HOUR_LABELS, MISSION_STEPS_MS)
  const hours = alignedTicks(start, end, step).map(
    (ms): AxisLabel => ({ label: tickLabel(ms), left: at(ms) }),
  )

  const rows = spans.map(
    (item): MissionTimelineRow => ({
      ...item,
      left: at(item.acceptedMs),
      width: at(item.expiryMs) - at(item.acceptedMs),
      elapsedWidth: at(Math.min(nowMs, item.expiryMs)) - at(item.acceptedMs),
      remainingSeconds: Math.max(0, Math.floor((item.expiryMs - nowMs) / 1000)),
    }),
  )

  return {
    start,
    end,
    hours,
    nowLeft: at(nowMs),
    etaLeft: etaMs === null || spans.length === 0 ? null : at(etaMs),
    rows,
  }
}

/**
 * Axis labels for the session timeline: the start time, evenly spaced UTC
 * ticks (quarter hours for short sessions, wider steps for long ones) that do
 * not crowd either edge, and the end time with seconds.
 */
export function sessionTimelineMarks(startMs: number, endMs: number): readonly AxisLabel[] {
  const span = Math.max(1, endMs - startMs)
  const at = (ms: number): number => ((ms - startMs) / span) * 100
  const step = axisStep(startMs + 1, endMs - 1, MAX_SESSION_INNER_LABELS, SESSION_STEPS_MS)
  const inner = alignedTicks(startMs + 1, endMs - 1, step)
    .filter((ms) => (ms - startMs) / span > 0.06 && (endMs - ms) / span > 0.12)
    .map((ms): AxisLabel => ({ label: tickLabel(ms), left: at(ms) }))
  return [
    { label: utcHourMinute(startMs), left: 0 },
    ...inner,
    { label: utcClock(endMs), left: 100 },
  ]
}

/** Number of step-aligned UTC ticks within [startMs, endMs]. */
function tickCount(startMs: number, endMs: number, stepMs: number): number {
  return Math.max(0, Math.floor(endMs / stepMs) - Math.ceil(startMs / stepMs) + 1)
}

/**
 * Smallest candidate step that keeps the tick count within `maxTicks`; whole
 * days beyond the largest candidate.
 */
function axisStep(
  startMs: number,
  endMs: number,
  maxTicks: number,
  candidates: readonly number[],
): number {
  for (const step of candidates) {
    if (tickCount(startMs, endMs, step) <= maxTicks) {
      return step
    }
  }
  return Math.max(1, Math.ceil((endMs - startMs) / maxTicks / DAY_MS)) * DAY_MS
}

function alignedTicks(startMs: number, endMs: number, stepMs: number): number[] {
  const ticks: number[] = []
  for (let ms = Math.ceil(startMs / stepMs) * stepMs; ms <= endMs; ms += stepMs) {
    ticks.push(ms)
  }
  return ticks
}

/** "HH:MM" for ticks within a day; the date ("21 Jun") at UTC midnight. */
function tickLabel(ms: number): string {
  if (ms % DAY_MS !== 0) {
    return utcHourMinute(ms)
  }
  const date = new Date(ms)
  return `${date.getUTCDate()} ${MONTHS[date.getUTCMonth()]}`
}

export type LabelSide = "left" | "right"

export type MarkerLabelLayout = {
  /** Side of its marker the label is drawn on. */
  readonly nowSide: LabelSide
  readonly etaSide: LabelSide
  /** True when the ETA label moves to a second line to avoid the now label. */
  readonly etaLow: boolean
}

/** Markers closer than this to an axis edge draw their label inward. */
const EDGE_PERCENT = 15
/** Approximate label width as a share of the axis. */
const LABEL_PERCENT = 16

/**
 * Placement of the "Now" and "ETA" labels on the mission timeline axis. By
 * default the now label sits before its marker and the ETA label after it;
 * near an edge a label flips inward so it is never clipped.
 */
export function markerLabelLayout(nowLeft: number, etaLeft: number | null): MarkerLabelLayout {
  const nowSide: LabelSide = nowLeft < EDGE_PERCENT ? "right" : "left"
  if (etaLeft === null) {
    return { nowSide, etaSide: "right", etaLow: false }
  }
  const etaSide: LabelSide = etaLeft > 100 - EDGE_PERCENT ? "left" : "right"
  const extent = (left: number, side: LabelSide): readonly [number, number] =>
    side === "left" ? [left - LABEL_PERCENT, left] : [left, left + LABEL_PERCENT]
  const [nowStart, nowEnd] = extent(nowLeft, nowSide)
  const [etaStart, etaEnd] = extent(etaLeft, etaSide)
  return { nowSide, etaSide, etaLow: nowStart < etaEnd && etaStart < nowEnd }
}
