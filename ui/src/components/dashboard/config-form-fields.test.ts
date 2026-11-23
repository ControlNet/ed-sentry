import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"
import {
  applyNumberFieldInput,
  createNumberFieldState,
  NumberFieldView,
  parseNumberFieldInput,
  syncNumberFieldState,
} from "./config-form-fields"

describe("NumberField empty and external sync behavior", () => {
  it("does not emit 0 when cleared, keeps the input empty, and propagates a typed number", () => {
    const onChange = vi.fn()
    let propValue = 20
    let state = createNumberFieldState(propValue)

    // User clears the input to retype a new value.
    const cleared = applyNumberFieldInput(state, "")
    state = cleared.state
    if (cleared.parsedValue !== null) {
      propValue = cleared.parsedValue
      onChange(cleared.parsedValue)
    }

    expect(onChange).not.toHaveBeenCalled()
    expect(propValue).toBe(20)
    expect(state.rawValue).toBe("")
    expect(parseNumberFieldInput(state.rawValue)).toBeNull()

    // Re-rendering with the unchanged external prop must keep the input empty instead of snapping to 0 or 20.
    state = syncNumberFieldState(state, propValue)
    expect(state.rawValue).toBe("")

    const emptyHtml = renderToStaticMarkup(
      createElement(NumberFieldView, {
        label: "No kills warning minutes",
        rawValue: state.rawValue,
        invalid: parseNumberFieldInput(state.rawValue) === null,
        min: 0,
        max: 1440,
        onRawChange: () => {},
      }),
    )
    expect(emptyHtml).toContain('value=""')
    expect(emptyHtml).toContain('aria-invalid="true"')

    // Typing a valid number propagates it and clears the invalid marker.
    const typed = applyNumberFieldInput(state, "35")
    state = typed.state
    if (typed.parsedValue !== null) {
      propValue = typed.parsedValue
      onChange(typed.parsedValue)
    }

    expect(onChange).toHaveBeenCalledTimes(1)
    expect(onChange).toHaveBeenCalledWith(35)
    state = syncNumberFieldState(state, propValue)
    expect(state.rawValue).toBe("35")

    const filledHtml = renderToStaticMarkup(
      createElement(NumberFieldView, {
        label: "No kills warning minutes",
        rawValue: state.rawValue,
        invalid: parseNumberFieldInput(state.rawValue) === null,
        min: 0,
        max: 1440,
        onRawChange: () => {},
      }),
    )
    expect(filledHtml).toContain('value="35"')
    expect(filledHtml).not.toContain("aria-invalid")

    // Typing an explicit "0" still propagates 0 when intended.
    const zeroed = applyNumberFieldInput(state, "0")
    expect(zeroed.parsedValue).toBe(0)

    // An external prop update resyncs even if the field was cleared.
    const clearedAgain = applyNumberFieldInput(state, "").state
    const resynced = syncNumberFieldState(clearedAgain, 45)
    expect(resynced).toEqual({ rawValue: "45", syncedValue: 45 })
  })
})
