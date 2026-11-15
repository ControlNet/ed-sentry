import { describe, expect, it } from "vitest"
import {
  formatCredits,
  formatDuration,
  formatMinutes,
  formatRate,
  parseTimestamp,
  utcClock,
  utcDateTime,
  utcHourMinute,
} from "./format"

describe("parseTimestamp", () => {
  it("parses RFC 3339 timestamps to epoch milliseconds", () => {
    expect(parseTimestamp("2026-06-20T14:11:19Z")).toBe(Date.UTC(2026, 5, 20, 14, 11, 19))
  })

  it("returns null for missing or invalid values", () => {
    expect(parseTimestamp(null)).toBeNull()
    expect(parseTimestamp(undefined)).toBeNull()
    expect(parseTimestamp("not a date")).toBeNull()
  })
})

describe("UTC clock formatting", () => {
  const at = Date.UTC(2026, 5, 20, 4, 5, 9)

  it("formats a time of day with seconds", () => {
    expect(utcClock(at)).toBe("04:05:09")
  })

  it("formats hours and minutes", () => {
    expect(utcHourMinute(at)).toBe("04:05")
  })

  it("formats a date with hours and minutes in UTC", () => {
    expect(utcDateTime(at)).toBe("2026-06-20 04:05 UTC")
  })
})

describe("formatDuration", () => {
  it("uses minutes and seconds below one hour", () => {
    expect(formatDuration(401)).toBe("6m 41s")
    expect(formatDuration(0)).toBe("0m 00s")
  })

  it("uses hours and minutes from one hour", () => {
    expect(formatDuration(3720)).toBe("1h 02m")
  })

  it("clamps negative values to zero", () => {
    expect(formatDuration(-5)).toBe("0m 00s")
  })
})

describe("formatMinutes", () => {
  it("drops seconds", () => {
    expect(formatMinutes(2520)).toBe("42m")
    expect(formatMinutes(59)).toBe("0m")
  })

  it("uses hours and minutes from one hour", () => {
    expect(formatMinutes(18_120)).toBe("5h 02m")
  })
})

describe("number formatting", () => {
  it("formats an hourly rate with one decimal", () => {
    expect(formatRate(31.24)).toBe("31.2/h")
  })

  it("formats credits with thousands separators", () => {
    expect(formatCredits(4_218_450)).toBe("4,218,450")
  })
})
