import { describe, expect, it } from "vitest"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import type { EventFeedItem } from "@/adapters/types"
import {
  eventCategory,
  eventCategoryCounts,
  eventLevelTone,
  eventTypeLabel,
  presentEventCategories,
} from "./events"

function item(source: string, eventType: string, level = 1): EventFeedItem {
  return {
    id: `${source}:${eventType}`,
    source,
    event_type: eventType,
    level,
    summary: eventType,
    timestamp: "2026-06-20T14:00:00Z",
    timestamp_display: "2026-06-20T14:00:00Z",
  }
}

describe("eventCategory", () => {
  it("groups notification event types", () => {
    expect(eventCategory(item("notification", "kill_bounty"))).toBe("kill")
    expect(eventCategory(item("notification", "kill_bond"))).toBe("kill")
    expect(eventCategory(item("notification", "cargo_scan"))).toBe("scan")
    expect(eventCategory(item("notification", "security_scan"))).toBe("scan")
    expect(eventCategory(item("notification", "mission_accepted"))).toBe("mission")
    expect(eventCategory(item("notification", "fighter_destroyed"))).toBe("ship")
    expect(eventCategory(item("notification", "fuel_report"))).toBe("ship")
    expect(eventCategory(item("notification", "cargo_lost"))).toBe("ship")
    expect(eventCategory(item("notification", "died"))).toBe("ship")
    expect(eventCategory(item("notification", "no_kills"))).toBe("warning")
    expect(eventCategory(item("notification", "kill_rate"))).toBe("warning")
    expect(eventCategory(item("notification", "merits"))).toBe("other")
  })

  it("uses the source for runtime lifecycle and warning events", () => {
    expect(eventCategory(item("lifecycle", "monitor_started"))).toBe("lifecycle")
    expect(eventCategory(item("warning", "runtime_warning"))).toBe("warning")
  })
})

describe("eventCategoryCounts", () => {
  it("counts the mock feed by category", () => {
    const counts = eventCategoryCounts(mockDashboardSnapshot.event_feed)
    expect(counts.get("kill")).toBe(1)
    expect(counts.get("mission")).toBe(1)
    expect(counts.get("scan")).toBe(1)
    expect(counts.get("lifecycle")).toBe(1)
    expect(counts.get("ship")).toBeUndefined()
  })

  it("lists only the categories present, in a fixed order", () => {
    const feed = [
      item("lifecycle", "monitor_started"),
      item("notification", "mission_status"),
      item("notification", "kill_bounty"),
    ]
    expect(presentEventCategories(feed)).toEqual(["kill", "mission", "lifecycle"])
  })
})

describe("event labels", () => {
  it("turns an event type into words", () => {
    expect(eventTypeLabel(item("notification", "kill_bounty"))).toBe("kill bounty")
  })

  it("maps notification levels to tones", () => {
    expect(eventLevelTone(0)).toBe("off")
    expect(eventLevelTone(1)).toBe("ok")
    expect(eventLevelTone(2)).toBe("warn")
    expect(eventLevelTone(3)).toBe("bad")
  })
})
