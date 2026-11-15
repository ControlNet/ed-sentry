import type { MissionView } from "@/adapters/types"
import { parseTimestamp, utcClock, utcHourMinute } from "./format"

const HOUR_MS = 3_600_000
const QUARTER_MS = 900_000
const MAX_HOUR_LABELS = 9

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

  const spanHours = span / HOUR_MS
  let step = 1
  while (Math.floor(spanHours / step) + 1 > MAX_HOUR_LABELS) {
    step += 1
  }
  const hours: AxisLabel[] = []
  for (let ms = start; ms <= end; ms += step * HOUR_MS) {
    hours.push({ label: utcHourMinute(ms), left: at(ms) })
  }

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
 * Axis labels for the session timeline: the start time, quarter hours that do
 * not crowd either edge, and the end time with seconds.
 */
export function sessionTimelineMarks(startMs: number, endMs: number): readonly AxisLabel[] {
  const span = Math.max(1, endMs - startMs)
  const at = (ms: number): number => ((ms - startMs) / span) * 100
  const marks: AxisLabel[] = [{ label: utcHourMinute(startMs), left: 0 }]
  for (let ms = Math.ceil((startMs + 1) / QUARTER_MS) * QUARTER_MS; ms < endMs; ms += QUARTER_MS) {
    if ((ms - startMs) / span > 0.06 && (endMs - ms) / span > 0.12) {
      marks.push({ label: utcHourMinute(ms), left: at(ms) })
    }
  }
  marks.push({ label: utcClock(endMs), left: 100 })
  return marks
}
