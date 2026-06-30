import type { EventFeedItem, MissionProgressView, ServiceStatusKind } from "@/adapters/dashboard"
import type { TacticalBadgeTone } from "./tactical-ui"

const statusGlyphLabels = new Map<string, string>([
  ["💥", "Kills"],
  ["📦", "Scans"],
  ["⌚", "uptime"],
  ["⏱", "uptime"],
  ["🎯", "missions"],
])
const pictographicGlyphPattern = /\p{Extended_Pictographic}/u

export function eventTitle(event: EventFeedItem): string {
  return lineSafeText(event.event_type).replaceAll("_", " ")
}

export function missionProgressLabel(progress: MissionProgressView): string {
  switch (progress.kind) {
    case "none":
      return "No progress tracked"
    case "massacre":
    case "trade":
      return progress.display
    default:
      return assertNever(progress)
  }
}

export function missionProgressPercent(progress: MissionProgressView): number {
  switch (progress.kind) {
    case "none":
      return 0
    case "massacre":
      return boundedPercent(progress.kills, progress.kill_count)
    case "trade":
      return boundedPercent(progress.delivered, progress.count)
    default:
      return assertNever(progress)
  }
}

export function sourceDetail(folder: string, selectedFile: string | null | undefined): string {
  const safeFolder = displaySafeText(folder)
  if (selectedFile === null || selectedFile === undefined) {
    return safeFolder
  }
  return joinJournalPath(safeFolder, displaySafeText(selectedFile))
}

export function statusTextClass(kind: ServiceStatusKind): string {
  switch (kind) {
    case "running":
      return "text-status-online"
    case "warning":
    case "starting":
      return "text-status-warning"
    case "error":
      return "text-status-danger"
    case "disabled":
      return "text-status-neutral"
    default:
      return assertNever(kind)
  }
}

export function serviceStatusBadgeTone(kind: ServiceStatusKind): TacticalBadgeTone {
  switch (kind) {
    case "running":
      return "success"
    case "warning":
    case "starting":
      return "warning"
    case "error":
      return "danger"
    case "disabled":
      return "default"
    default:
      return assertNever(kind)
  }
}

export function assertNever(value: never): never {
  throw new Error(`Unhandled dashboard variant: ${String(value)}`)
}

export function lineSafeText(value: string): string {
  return displaySafeText(value).replaceAll(/Journal\.[^\s/\\]+\.log/g, "Selected Journal file")
}

export function fuelSummary(events: readonly EventFeedItem[]): string {
  const fuelEvent = events.find((event) => event.event_type === "fuel_report")
  return fuelEvent === undefined ? "Fuel telemetry unavailable" : lineSafeText(fuelEvent.summary)
}

export function healthToneClass(percent: number | null | undefined): string {
  if (percent === null || percent === undefined) {
    return "bg-status-neutral"
  }
  if (percent <= 0.35) {
    return "bg-status-danger"
  }
  if (percent <= 0.7) {
    return "bg-status-warning"
  }
  return "bg-status-online"
}

export function healthPercent(percent: number | null | undefined): number {
  if (percent === null || percent === undefined) {
    return 0
  }
  return Math.min(100, Math.max(0, Math.round(percent * 100)))
}

function boundedPercent(value: number, total: number): number {
  if (total <= 0) {
    return 0
  }
  return Math.min(100, Math.max(0, Math.round((value / total) * 100)))
}

function displaySafeCharacter(character: string): string {
  const statusLabel = statusGlyphLabels.get(character)
  if (statusLabel !== undefined) {
    return `${statusLabel} `
  }

  const codePoint = character.codePointAt(0)
  if (
    codePoint === undefined ||
    isLineUnsafeControl(codePoint) ||
    isEmojiSequenceMarker(codePoint) ||
    isStatusGlyph(codePoint, character)
  ) {
    return " "
  }
  return character
}

function isLineUnsafeControl(codePoint: number): boolean {
  return (
    codePoint === 9 || codePoint === 10 || codePoint === 13 || codePoint === 27 || codePoint === 155
  )
}

function isEmojiSequenceMarker(codePoint: number): boolean {
  return (
    codePoint === 0x200d ||
    codePoint === 0x20e3 ||
    codePoint === 0xfe0e ||
    codePoint === 0xfe0f ||
    (codePoint >= 0x1f3fb && codePoint <= 0x1f3ff) ||
    (codePoint >= 0x1f1e6 && codePoint <= 0x1f1ff) ||
    (codePoint >= 0xe0020 && codePoint <= 0xe007f)
  )
}

function isStatusGlyph(codePoint: number, character: string): boolean {
  return (
    pictographicGlyphPattern.test(character) ||
    (codePoint >= 0x2300 && codePoint <= 0x23ff) ||
    (codePoint >= 0x2600 && codePoint <= 0x27bf) ||
    (codePoint >= 0x1f000 && codePoint <= 0x1faff)
  )
}

export function displaySafeText(value: string): string {
  return Array.from(value, displaySafeCharacter)
    .join("")
    .replaceAll(/\s{2,}/g, " ")
    .trim()
}

function joinJournalPath(folder: string, selectedFile: string): string {
  if (folder === "") {
    return selectedFile
  }
  if (folder === ".") {
    return `.${pathSeparator(selectedFile)}${selectedFile}`
  }
  const separator = pathSeparator(folder)
  if (folder.endsWith("/") || folder.endsWith("\\")) {
    return `${folder}${selectedFile}`
  }
  return `${folder}${separator}${selectedFile}`
}

function pathSeparator(path: string): "/" | "\\" {
  return /^[A-Za-z]:\\/.test(path) || path.includes("\\") ? "\\" : "/"
}
