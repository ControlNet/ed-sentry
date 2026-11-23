import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"
import type { DashboardAdapter } from "@/adapters/dashboard"
import { loadMockConfigView, saveMockConfigView } from "@/adapters/mock-config"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { formFromConfig, isConfigFormDirty, updateFromForm } from "./config-form-model"
import {
  AUTOSAVE_DELAY_MS,
  ConfigPanelView,
  canScheduleAutosave,
  classifySaveFailure,
  pickActiveSection,
  type SaveState,
} from "./config-panel"

function createStubAdapter(saveConfig: DashboardAdapter["saveConfig"]): DashboardAdapter {
  return {
    mode: "web",
    label: "Web API",
    loadSnapshot: async () => mockDashboardSnapshot,
    loadConfig: async () => loadMockConfigView("default"),
    saveConfig,
    subscribe: () => () => undefined,
  }
}

describe("ConfigPanel autosave error handling and tunnel re-auth", () => {
  it("makes exactly one save request after a failure across multiple timer ticks and renders manual Retry", async () => {
    vi.useFakeTimers()
    try {
      const view = loadMockConfigView("default")
      const savedForm = formFromConfig(view.config)
      const editedForm = {
        ...savedForm,
        monitor: {
          ...savedForm.monitor,
          warn_kill_rate: savedForm.monitor.warn_kill_rate + 5,
        },
      }
      expect(isConfigFormDirty(editedForm, savedForm)).toBe(true)

      const saveConfig = vi.fn(async () => {
        throw new Error("The config file could not be saved.")
      })
      const adapter = createStubAdapter(saveConfig)

      let saveState: SaveState = "pending"
      let saveMessage: string | null = "Autosave pending"
      let activeTimer: ReturnType<typeof setTimeout> | null = null

      const scheduleIfEligible = (): void => {
        if (activeTimer !== null) {
          clearTimeout(activeTimer)
          activeTimer = null
        }
        const canSave = saveState !== "saving"
        if (!canScheduleAutosave(canSave, false, saveState)) {
          return
        }
        activeTimer = setTimeout(() => {
          activeTimer = null
          saveState = "saving"
          void adapter
            .saveConfig(updateFromForm(editedForm))
            .then(() => {
              saveState = "saved"
              scheduleIfEligible()
            })
            .catch((error: unknown) => {
              const failure = classifySaveFailure(error)
              saveState = failure.kind === "auth_required" ? "idle" : "error"
              saveMessage = failure.message
              scheduleIfEligible()
            })
        }, AUTOSAVE_DELAY_MS)
      }

      // First edit schedules one autosave.
      scheduleIfEligible()
      await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY_MS)
      expect(saveConfig).toHaveBeenCalledTimes(1)
      expect(saveState).toBe("error")

      // Advancing several more autosave windows must NOT trigger any further requests.
      await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY_MS * 5)
      expect(saveConfig).toHaveBeenCalledTimes(1)
      expect(canScheduleAutosave(true, false, saveState)).toBe(false)

      const errorHtml = renderToStaticMarkup(
        createElement(ConfigPanelView, {
          adapter,
          state: { status: "ready", view, form: editedForm, savedForm },
          saveState,
          saveMessage,
          restartNotice: null,
          validationErrors: [],
          protectedChange: false,
          canSave: true,
          activeSection: "monitor",
          onJumpToSection: () => {},
          onFormChange: () => {},
          onSaveProtectedChange: () => {},
          onRetrySave: () => {},
          onAuthenticated: () => {},
        }),
      )
      expect(errorHtml).toContain("The config file could not be saved.")
      expect(errorHtml).toContain(">Retry<")
      expect(errorHtml).not.toMatch(/<button[^>]*disabled[^>]*>Retry<\/button>/)

      // A subsequent user edit marks the form pending again and schedules one new save.
      saveState = "pending"
      scheduleIfEligible()
      await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY_MS)
      expect(saveConfig).toHaveBeenCalledTimes(2)
    } finally {
      vi.useRealTimers()
    }
  })

  it("switches to TunnelConfigAuthPrompt when save fails with tunnel_config_auth_required", () => {
    const adapter = createStubAdapter(async (update) => saveMockConfigView(update))
    const failure = classifySaveFailure(
      new Error("Tunnel config access requires a valid bearer token"),
    )

    expect(failure).toEqual({
      kind: "auth_required",
      message: "Tunnel config access requires a valid bearer token",
    })

    const html = renderToStaticMarkup(
      createElement(ConfigPanelView, {
        adapter,
        state: { status: "error", message: failure.message },
        saveState: "idle",
        saveMessage: null,
        restartNotice: null,
        validationErrors: [],
        protectedChange: false,
        canSave: false,
        activeSection: "journal",
        onJumpToSection: () => {},
        onFormChange: () => {},
        onSaveProtectedChange: () => {},
        onRetrySave: () => {},
        onAuthenticated: () => {},
      }),
    )

    expect(html).toContain('aria-label="Tunnel config authentication"')
    expect(html).toContain("Unlock Systems")
    expect(html).not.toContain("Config unavailable")
  })
})

describe("pickActiveSection", () => {
  const sections = [
    { id: "journal", top: -400 },
    { id: "matrix", top: -120 },
    { id: "tunnel", top: 180 },
    { id: "web", top: 520 },
    { id: "monitor", top: 900 },
    { id: "routing", top: 1400 },
  ] as const

  it("selects the last section whose heading has reached the top of the pane", () => {
    expect(pickActiveSection(sections, false)).toBe("matrix")
  })

  it("treats a heading just below the top edge as reached", () => {
    const nearTop = sections.map((section) =>
      section.id === "tunnel" ? { ...section, top: 12 } : section,
    )
    expect(pickActiveSection(nearTop, false)).toBe("tunnel")
  })

  it("falls back to the first section before any heading reaches the top", () => {
    const unscrolled = sections.map((section) => ({ ...section, top: section.top + 600 }))
    expect(pickActiveSection(unscrolled, false)).toBe("journal")
  })

  it("selects the last section once the pane is scrolled to the bottom", () => {
    expect(pickActiveSection(sections, true)).toBe("routing")
  })
})
