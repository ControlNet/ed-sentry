import { describe, expect, it } from "vitest"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import type { MissionView } from "@/adapters/types"
import {
  formatTrend,
  liveNowMs,
  missionEtaLabel,
  missionEtaMinutes,
  noKillsProgress,
  rateAxisLabels,
  rateDisplay,
  rateScaleMax,
  readinessSummary,
  scalePercent,
  segmentsOn,
  sharedKillProgress,
  trendPercent,
} from "./telemetry"

function mockMission(index: number): MissionView {
  const mission = mockDashboardSnapshot.missions.items[index]
  if (mission === undefined) {
    throw new Error(`mock mission ${index} missing`)
  }
  return mission
}

const first = mockMission(0)
const second = mockMission(1)

function massacre(
  base: MissionView,
  overrides: { issuing?: string; target?: string; kills: number; count: number },
): MissionView {
  return {
    ...base,
    mission_id: base.mission_id + overrides.kills + overrides.count,
    issuing_faction: overrides.issuing ?? base.issuing_faction ?? null,
    progress: {
      kind: "massacre",
      target: "Pirates",
      target_faction: overrides.target ?? "Raiders",
      kills: overrides.kills,
      kill_count: overrides.count,
      display: `${overrides.kills}/${overrides.count} kills`,
    },
  }
}

describe("rate comparison", () => {
  it("scales a value against a maximum and clamps to 0..100", () => {
    expect(scalePercent(10, 40)).toBe(25)
    expect(scalePercent(60, 40)).toBe(100)
    expect(scalePercent(-1, 40)).toBe(0)
    expect(scalePercent(5, 0)).toBe(0)
  })

  it("keeps the base scale until a value exceeds it, then rounds up by half the base", () => {
    expect(rateScaleMax([31.2, 25.7, 20], 40)).toBe(40)
    expect(rateScaleMax([41, 25.7], 40)).toBe(60)
    expect(rateScaleMax([10.4], 10)).toBe(15)
  })

  it("computes the recent-versus-average trend in whole percent", () => {
    expect(trendPercent(31.2, 25.7)).toBe(21)
    expect(trendPercent(7.9, 8.6)).toBe(-8)
  })

  it("has no trend without a positive average", () => {
    expect(trendPercent(5, 0)).toBeNull()
  })

  it("formats the trend with an arrow and sign", () => {
    expect(formatTrend(21)).toBe("▲ +21%")
    expect(formatTrend(-8)).toBe("▼ −8%")
    expect(formatTrend(0)).toBe("▲ +0%")
  })
})

describe("rateDisplay", () => {
  it("formats a rate once events exist", () => {
    expect(rateDisplay(18, 25.66)).toBe("25.7/h")
  })

  it("uses the CLI idle marker before the first event", () => {
    expect(rateDisplay(0, 0)).toBe("-/h")
  })
})

describe("noKillsProgress", () => {
  const lastKill = Date.UTC(2026, 5, 20, 14, 11, 19)

  it("measures the time since the last kill against the warning threshold", () => {
    const now = lastKill + 401_000
    expect(noKillsProgress(lastKill, now, 20)).toEqual({
      seconds: 401,
      fraction: 401 / 1200,
      over: false,
    })
  })

  it("is over at or above the threshold and caps the fraction at 1", () => {
    expect(noKillsProgress(lastKill, lastKill + 1_200_000, 20).over).toBe(true)
    expect(noKillsProgress(lastKill, lastKill + 3_000_000, 20).fraction).toBe(1)
  })

  it("never reports negative elapsed time", () => {
    expect(noKillsProgress(lastKill, lastKill - 5_000, 20).seconds).toBe(0)
  })

  it("has no fraction when the warning is disabled", () => {
    expect(noKillsProgress(lastKill, lastKill + 60_000, 0)).toEqual({
      seconds: 60,
      fraction: null,
      over: false,
    })
  })
})

describe("segmentsOn", () => {
  it("rounds a fraction to lit segments", () => {
    expect(segmentsOn(0.92, 25)).toBe(23)
    expect(segmentsOn(0.76, 25)).toBe(19)
  })

  it("treats missing or out-of-range values as bounded", () => {
    expect(segmentsOn(null, 25)).toBe(0)
    expect(segmentsOn(1.4, 25)).toBe(25)
    expect(segmentsOn(-0.2, 25)).toBe(0)
  })
})

describe("sharedKillProgress", () => {
  it("counts the stack by the largest issuer requirement per target faction", () => {
    expect(sharedKillProgress(mockDashboardSnapshot.missions.items)).toEqual({
      completed: 24,
      total: 36,
      remaining: 12,
    })
  })

  it("sums missions from the same issuer before taking the maximum", () => {
    const missions = [
      massacre(first, { issuing: "A", kills: 5, count: 10 }),
      massacre(first, { issuing: "A", kills: 0, count: 10 }),
      massacre(second, { issuing: "B", kills: 3, count: 12 }),
    ]
    expect(sharedKillProgress(missions)).toEqual({ completed: 5, total: 20, remaining: 15 })
  })

  it("adds separate target factions", () => {
    const missions = [
      massacre(first, { target: "X", kills: 2, count: 4 }),
      massacre(second, { target: "Y", kills: 1, count: 6 }),
    ]
    expect(sharedKillProgress(missions)).toEqual({ completed: 3, total: 10, remaining: 7 })
  })

  it("ignores missions without kill progress", () => {
    expect(sharedKillProgress([{ ...first, progress: { kind: "none" } }])).toEqual({
      completed: 0,
      total: 0,
      remaining: 0,
    })
  })
})

describe("mission ETA", () => {
  it("estimates whole minutes from the session kill rate", () => {
    expect(missionEtaMinutes(12, 25.7)).toBe(28)
  })

  it("is zero when nothing remains and unknown without a rate", () => {
    expect(missionEtaMinutes(0, 25.7)).toBe(0)
    expect(missionEtaMinutes(5, 0)).toBeNull()
  })

  it("formats minutes, hours and completion", () => {
    expect(missionEtaLabel(12, 25.7)).toBe("28m")
    expect(missionEtaLabel(60, 20)).toBe("3h")
    expect(missionEtaLabel(50, 20)).toBe("2h 30m")
    expect(missionEtaLabel(0, 20)).toBe("Complete")
    expect(missionEtaLabel(4, 0)).toBe("—")
  })
})

describe("readinessSummary", () => {
  it("counts passing checks", () => {
    expect(readinessSummary(mockDashboardSnapshot.afk_checklist.rows)).toEqual({
      pass: 2,
      total: 3,
    })
  })
})

describe("liveNowMs", () => {
  it("advances the snapshot time by the local time elapsed since it was received", () => {
    const generated = Date.UTC(2026, 5, 20, 14, 18, 0)
    expect(liveNowMs(generated, 1_000_000, 1_004_500)).toBe(generated + 4_500)
  })

  it("never moves before the snapshot time", () => {
    const generated = Date.UTC(2026, 5, 20, 14, 18, 0)
    expect(liveNowMs(generated, 1_000_000, 999_000)).toBe(generated)
  })
})

describe("rateAxisLabels", () => {
  it("labels zero, half scale and the maximum without a warning threshold", () => {
    expect(rateAxisLabels(40, null)).toEqual([
      { text: "0", left: 0, align: "start", warn: false },
      { text: "20", left: 50, align: "center", warn: false },
      { text: "40/h", left: 100, align: "end", warn: false },
    ])
  })

  it("labels the warning threshold at its position on the scale", () => {
    expect(rateAxisLabels(40, 20)).toEqual([
      { text: "0", left: 0, align: "start", warn: false },
      { text: "20", left: 50, align: "center", warn: true },
      { text: "40/h", left: 100, align: "end", warn: false },
    ])
  })

  it("replaces the zero label when the threshold sits next to it", () => {
    const labels = rateAxisLabels(7200, 20)
    expect(labels.map((label) => label.text)).toEqual(["20", "7200/h"])
    expect(labels[0]).toMatchObject({ align: "start", warn: true })
  })

  it("replaces the maximum label when the threshold sits next to it", () => {
    expect(rateAxisLabels(40, 40)).toEqual([
      { text: "0", left: 0, align: "start", warn: false },
      { text: "40/h", left: 100, align: "end", warn: true },
    ])
  })
})
