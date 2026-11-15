import type { EventFeedItem } from "@/adapters/types"
import { lineSafeText } from "@/components/dashboard/dashboard-helpers"

export const eventCategories = [
  "kill",
  "scan",
  "mission",
  "ship",
  "warning",
  "lifecycle",
  "other",
] as const

export type EventCategory = (typeof eventCategories)[number]

export const eventCategoryLabels: Readonly<Record<EventCategory, string>> = {
  kill: "Kill",
  scan: "Scan",
  mission: "Mission",
  ship: "Ship",
  warning: "Warning",
  lifecycle: "Lifecycle",
  other: "Other",
}

const warningTypes = new Set(["no_kills", "kill_rate", "runtime_warning"])
const shipTypes = new Set(["cargo_lost", "died"])
const shipPrefixes = ["ship_", "fighter_", "fuel_", "shipyard_"]

export function eventCategory(item: EventFeedItem): EventCategory {
  if (item.source === "lifecycle") {
    return "lifecycle"
  }
  const type = item.event_type
  if (item.source === "warning" || warningTypes.has(type)) {
    return "warning"
  }
  if (type.startsWith("kill")) {
    return "kill"
  }
  if (type.includes("scan")) {
    return "scan"
  }
  if (type.startsWith("mission")) {
    return "mission"
  }
  if (shipTypes.has(type) || shipPrefixes.some((prefix) => type.startsWith(prefix))) {
    return "ship"
  }
  return "other"
}

export function eventCategoryCounts(items: readonly EventFeedItem[]): Map<EventCategory, number> {
  const counts = new Map<EventCategory, number>()
  for (const item of items) {
    const category = eventCategory(item)
    counts.set(category, (counts.get(category) ?? 0) + 1)
  }
  return counts
}

export function presentEventCategories(items: readonly EventFeedItem[]): readonly EventCategory[] {
  const counts = eventCategoryCounts(items)
  return eventCategories.filter((category) => counts.has(category))
}

export function eventTypeLabel(item: EventFeedItem): string {
  return lineSafeText(item.event_type).replaceAll("_", " ")
}

export type Tone = "ok" | "warn" | "bad" | "off"

export function eventLevelTone(level: number): Tone {
  if (level >= 3) {
    return "bad"
  }
  if (level === 2) {
    return "warn"
  }
  if (level === 1) {
    return "ok"
  }
  return "off"
}
