// Display formatting for snapshot values. Clock values are shown in UTC, the
// same clock the game uses for Journal timestamps.

const pad2 = (value: number): string => String(value).padStart(2, "0")

export function parseTimestamp(value: string | null | undefined): number | null {
  if (value === null || value === undefined) {
    return null
  }
  const parsed = Date.parse(value)
  return Number.isNaN(parsed) ? null : parsed
}

export function utcClock(epochMs: number): string {
  const date = new Date(epochMs)
  return `${pad2(date.getUTCHours())}:${pad2(date.getUTCMinutes())}:${pad2(date.getUTCSeconds())}`
}

export function utcHourMinute(epochMs: number): string {
  const date = new Date(epochMs)
  return `${pad2(date.getUTCHours())}:${pad2(date.getUTCMinutes())}`
}

export function utcDateTime(epochMs: number): string {
  const date = new Date(epochMs)
  const day = `${date.getUTCFullYear()}-${pad2(date.getUTCMonth() + 1)}-${pad2(date.getUTCDate())}`
  return `${day} ${utcHourMinute(epochMs)} UTC`
}

/** Elapsed time with seconds below one hour ("6m 41s"), else hours and minutes ("1h 02m"). */
export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds))
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor(total / 60) % 60
  if (hours > 0) {
    return `${hours}h ${pad2(minutes)}m`
  }
  return `${minutes}m ${pad2(total % 60)}s`
}

/** Elapsed time without seconds ("42m", "5h 02m"). */
export function formatMinutes(seconds: number): string {
  const minutes = Math.floor(Math.max(0, seconds) / 60)
  if (minutes >= 60) {
    return `${Math.floor(minutes / 60)}h ${pad2(minutes % 60)}m`
  }
  return `${minutes}m`
}

export function formatRate(perHour: number): string {
  return `${perHour.toFixed(1)}/h`
}

export function formatCredits(value: number): string {
  return Math.round(value).toLocaleString("en-US")
}
