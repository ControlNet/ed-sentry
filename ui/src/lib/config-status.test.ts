import { describe, expect, it } from "vitest"
import { logLevelKeys } from "@/adapters/config"
import { loadMockConfigView } from "@/adapters/mock-config"
import { formFromConfig } from "@/components/dashboard/config-form-model"
import { configSectionStatus, routingGroups, routingSummary } from "./config-status"

const form = formFromConfig(loadMockConfigView("default").config)

describe("routingSummary", () => {
  it("counts events, mentions and disabled routes", () => {
    const levels = { ...form.log_levels, merits: 0, died: 2, no_kills: 2 }
    const summary = routingSummary(levels)
    expect(summary.total).toBe(logLevelKeys.length)
    expect(summary.mention).toBe(Object.values(levels).filter((value) => value >= 2).length)
    expect(summary.off).toBe(Object.values(levels).filter((value) => value <= 0).length)
  })
})

describe("routingGroups", () => {
  it("assigns every log level key to exactly one group", () => {
    const keys = routingGroups.flatMap((group) => group.keys)
    expect([...keys].sort()).toEqual([...logLevelKeys].sort())
  })
})

describe("configSectionStatus", () => {
  it("summarises each section from the current form", () => {
    const status = configSectionStatus({
      ...form,
      journal: { folder: "", recent_files: 10 },
      matrix: { ...form.matrix, enabled: false },
      tunnel: { ...form.tunnel, auto_start: false },
      web: { ...form.web, enabled: true, host: "127.0.0.1", port: 8765 },
      monitor: { ...form.monitor, warn_kill_rate: 20, warn_no_kills_minutes: 20 },
    })
    expect(status.journal).toEqual({ text: "Default folder · 10 recent files", on: true })
    expect(status.matrix).toEqual({ text: "Disabled", on: false })
    expect(status.tunnel).toEqual({ text: "Cloudflare · manual start", on: true })
    expect(status.web).toEqual({ text: "127.0.0.1:8765 · enabled", on: true })
    expect(status.monitor).toEqual({ text: "Warn 20/h · no kills 20 min", on: true })
    expect(status.routing.on).toBe(true)
    expect(status.routing.text).toMatch(/^29 events · \d+ mention · \d+ off$/)
  })

  it("reports custom folders, auto start and disabled web", () => {
    const status = configSectionStatus({
      ...form,
      journal: { folder: "D:/Journals", recent_files: 3 },
      matrix: { ...form.matrix, enabled: true },
      tunnel: { ...form.tunnel, auto_start: true },
      web: { ...form.web, enabled: false, host: "0.0.0.0", port: 9000 },
    })
    expect(status.journal.text).toBe("Custom folder · 3 recent files")
    expect(status.matrix).toEqual({ text: "Enabled", on: true })
    expect(status.tunnel.text).toBe("Cloudflare · auto start")
    expect(status.web).toEqual({ text: "0.0.0.0:9000 · disabled", on: false })
  })
})
