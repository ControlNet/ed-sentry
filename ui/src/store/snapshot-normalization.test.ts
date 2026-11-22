import { describe, expect, it } from "vitest"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { shouldApplySnapshotUpdate } from "./snapshot-normalization"

describe("shouldApplySnapshotUpdate", () => {
  it("ignores a snapshot that only has a newer generation time", () => {
    const next = { ...mockDashboardSnapshot, generated_at: "2026-06-20T14:19:00Z" }
    expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, next)).toBe(false)
  })

  it("applies a snapshot whose warning thresholds changed", () => {
    const next = {
      ...mockDashboardSnapshot,
      thresholds: { warn_kill_rate_per_hour: 15, warn_no_kills_minutes: 20 },
    }
    expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, next)).toBe(true)
  })

  it("applies a snapshot that differs only in event_feed", () => {
    const next = {
      ...mockDashboardSnapshot,
      event_feed: [
        {
          id: "warning:runtime_warning:1781965140000:000000000001",
          source: "warning",
          event_type: "runtime_warning",
          level: 2,
          summary: "Malformed journal line 9: invalid timestamp",
          timestamp: "2026-06-20T14:19:00Z",
          timestamp_display: "2026-06-20T14:19:00Z",
        },
        ...mockDashboardSnapshot.event_feed,
      ],
    }
    expect(shouldApplySnapshotUpdate(mockDashboardSnapshot, next)).toBe(true)
  })
})
