import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"
import { loadMockConfigView } from "@/adapters/mock-config"
import { formFromConfig, validateConfigForm } from "./config-form-model"
import { ConfigMonitorSection } from "./config-monitor-section"

const baseForm = formFromConfig(loadMockConfigView("default").config)

describe("validateConfigForm monitor warning bounds", () => {
  it("accepts zero for no-kills warning thresholds and rejects zero cooldown", () => {
    const allowed = validateConfigForm({
      ...baseForm,
      monitor: {
        ...baseForm.monitor,
        warn_no_kills_minutes: 0,
        warn_no_kills_initial_minutes: 0,
        warn_kill_rate_delay_minutes: 0,
        warn_cooldown_minutes: 1,
      },
    })
    expect(allowed).toEqual([])

    const rejected = validateConfigForm({
      ...baseForm,
      monitor: {
        ...baseForm.monitor,
        warn_cooldown_minutes: 0,
      },
    })
    expect(rejected).toContain("Warning cooldown minutes must be between 1 and 1440.")
  })

  it("rejects minute thresholds above 1440 matching backend validation", () => {
    const errors = validateConfigForm({
      ...baseForm,
      monitor: {
        ...baseForm.monitor,
        warn_kill_rate_delay_minutes: 1441,
        warn_no_kills_minutes: 1441,
        warn_no_kills_initial_minutes: 1441,
        warn_cooldown_minutes: 1441,
      },
    })
    expect(errors).toEqual([
      "Warn kill rate delay minutes must be between 0 and 1440.",
      "No kills warning minutes must be between 0 and 1440.",
      "Initial no kills warning minutes must be between 0 and 1440.",
      "Warning cooldown minutes must be between 1 and 1440.",
    ])
  })
})

describe("ConfigMonitorSection", () => {
  it("renders warning cooldown input with min=1", () => {
    const html = renderToStaticMarkup(
      createElement(ConfigMonitorSection, {
        form: baseForm,
        onChange: () => {},
      }),
    )
    expect(html).toContain('aria-label="Warning cooldown minutes"')
    expect(html).toMatch(/aria-label="Warning cooldown minutes"[^>]*min="1"/)
  })
})
