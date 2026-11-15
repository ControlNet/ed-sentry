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
})
