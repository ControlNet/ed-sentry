import { describe, expect, it } from "vitest"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { markerLabelLayout, missionTimeline, sessionTimelineMarks } from "./timeline"

const at = (h: number, m: number, s = 0): number => Date.UTC(2026, 5, 20, h, m, s)

describe("missionTimeline", () => {
  const now = at(14, 18)
  const eta = at(14, 46)
  const timeline = missionTimeline(mockDashboardSnapshot.missions.items, now, eta)

  it("spans whole hours from the earliest acceptance to the latest expiry", () => {
    expect(timeline.start).toBe(at(13, 0))
    expect(timeline.end).toBe(at(21, 0))
  })

  it("labels each hour while keeping at most nine labels", () => {
    expect(timeline.hours.map((hour) => hour.label)).toEqual([
      "13:00",
      "14:00",
      "15:00",
      "16:00",
      "17:00",
      "18:00",
      "19:00",
      "20:00",
      "21:00",
    ])
    expect(timeline.hours[0]?.left).toBe(0)
    expect(timeline.hours[8]?.left).toBe(100)
  })

  it("places now and the stack ETA on the shared axis", () => {
    expect(timeline.nowLeft).toBeCloseTo((78 / 480) * 100, 5)
    expect(timeline.etaLeft).toBeCloseTo((106 / 480) * 100, 5)
  })

  it("draws each mission from acceptance to expiry with elapsed time filled", () => {
    const row = timeline.rows[0]
    expect(row?.missionId).toBe(410024)
    expect(row?.left).toBeCloseTo((20 / 480) * 100, 5)
    expect(row?.width).toBeCloseTo((360 / 480) * 100, 5)
    expect(row?.elapsedWidth).toBeCloseTo((58 / 480) * 100, 5)
    expect(row?.remainingSeconds).toBe(5 * 3600 + 2 * 60)
  })

  it("skips missions without an expiry", () => {
    const [mission] = mockDashboardSnapshot.missions.items
    if (mission === undefined) {
      throw new Error("mock mission missing")
    }
    const result = missionTimeline([{ ...mission, expiry: null }], now, null)
    expect(result.rows).toEqual([])
    expect(result.etaLeft).toBeNull()
  })

  it("uses a wider whole-hour step aligned to UTC for long spans", () => {
    const [mission] = mockDashboardSnapshot.missions.items
    if (mission === undefined) {
      throw new Error("mock mission missing")
    }
    const long = { ...mission, expiry: "2026-06-21T13:20:00Z" }
    const result = missionTimeline([long], now, null)
    expect(result.hours.map((hour) => hour.label)).toEqual([
      "15:00",
      "18:00",
      "21:00",
      "21 Jun",
      "03:00",
      "06:00",
      "09:00",
      "12:00",
    ])
  })

  it("labels days for missions that run for several days", () => {
    const [mission] = mockDashboardSnapshot.missions.items
    if (mission === undefined) {
      throw new Error("mock mission missing")
    }
    const multiDay = { ...mission, expiry: "2026-06-26T13:20:00Z" }
    const result = missionTimeline([multiDay], now, null)
    expect(result.hours.map((hour) => hour.label)).toEqual([
      "21 Jun",
      "22 Jun",
      "23 Jun",
      "24 Jun",
      "25 Jun",
      "26 Jun",
    ])
  })

  it("keeps the label count bounded for very long spans", () => {
    const [mission] = mockDashboardSnapshot.missions.items
    if (mission === undefined) {
      throw new Error("mock mission missing")
    }
    const result = missionTimeline([{ ...mission, expiry: "2035-01-04T12:32:00Z" }], now, null)
    expect(result.hours.length).toBeGreaterThan(1)
    expect(result.hours.length).toBeLessThanOrEqual(9)
  })
})

describe("sessionTimelineMarks", () => {
  it("labels the start, quarter hours that fit, and the end", () => {
    const marks = sessionTimelineMarks(at(13, 36), at(14, 18))
    expect(marks.map((mark) => mark.label)).toEqual(["13:36", "13:45", "14:00", "14:18:00"])
    expect(marks[0]?.left).toBe(0)
    expect(marks.at(-1)?.left).toBe(100)
  })

  it("drops quarter-hour labels too close to either edge", () => {
    const marks = sessionTimelineMarks(at(13, 44), at(14, 2))
    expect(marks.map((mark) => mark.label)).toEqual(["13:44", "14:02:00"])
  })

  it("keeps long AFK sessions to a few evenly spaced labels", () => {
    const marks = sessionTimelineMarks(at(8, 10), at(16, 25))
    expect(marks.map((mark) => mark.label)).toEqual([
      "08:10",
      "10:00",
      "12:00",
      "14:00",
      "16:25:00",
    ])
  })

  it("stays bounded for spans of many days", () => {
    const marks = sessionTimelineMarks(at(8, 10), Date.UTC(2035, 0, 3, 10, 3))
    expect(marks.length).toBeLessThanOrEqual(8)
  })
})

describe("markerLabelLayout", () => {
  it("puts the now label before its marker and the ETA label after its marker", () => {
    expect(markerLabelLayout(40, 70)).toEqual({ nowSide: "left", etaSide: "right", etaLow: false })
  })

  it("keeps labels inside the axis when a marker sits near an edge", () => {
    expect(markerLabelLayout(0.2, 60)).toEqual({
      nowSide: "right",
      etaSide: "right",
      etaLow: false,
    })
    expect(markerLabelLayout(50, 99)).toEqual({ nowSide: "left", etaSide: "left", etaLow: false })
  })

  it("moves the ETA label to a second line when the two labels would overlap", () => {
    expect(markerLabelLayout(1, 8)).toEqual({ nowSide: "right", etaSide: "right", etaLow: true })
    expect(markerLabelLayout(90, 95)).toEqual({ nowSide: "left", etaSide: "left", etaLow: true })
  })

  it("leaves the ETA label alone when there is no ETA", () => {
    expect(markerLabelLayout(5, null)).toEqual({
      nowSide: "right",
      etaSide: "right",
      etaLow: false,
    })
  })
})
