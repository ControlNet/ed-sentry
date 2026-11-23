import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"
import {
  isTunnelUnlockDisabled,
  runTunnelUnlock,
  TunnelConfigAuthPromptView,
  type TunnelUnlockState,
} from "./tunnel-config-auth-prompt"

describe("TunnelConfigAuthPrompt unlock error recovery", () => {
  it("shows error message and re-enables unlock button when loginTunnel rejects", async () => {
    const states: TunnelUnlockState[] = []
    const onSuccess = vi.fn()
    const loginTunnel = vi.fn(async () => {
      throw new Error("Network unreachable")
    })

    await runTunnelUnlock(loginTunnel, "fixture-password", {
      onStateChange: (state) => {
        states.push(state)
      },
      onSuccess,
    })

    expect(onSuccess).not.toHaveBeenCalled()
    expect(states).toEqual([
      { status: "submitting" },
      { status: "error", message: "Network unreachable" },
    ])

    const [submittingState, finalState] = states
    if (submittingState === undefined || finalState === undefined) {
      throw new Error("Expected submitting and error states")
    }

    const submittingHtml = renderToStaticMarkup(
      createElement(TunnelConfigAuthPromptView, {
        password: "fixture-password",
        loginState: submittingState,
        onPasswordChange: () => {},
        onSubmit: () => {},
      }),
    )
    expect(submittingHtml).toMatch(/<button[^>]*disabled=""/)

    expect(isTunnelUnlockDisabled(finalState, "fixture-password")).toBe(false)

    const errorHtml = renderToStaticMarkup(
      createElement(TunnelConfigAuthPromptView, {
        password: "fixture-password",
        loginState: finalState,
        onPasswordChange: () => {},
        onSubmit: () => {},
      }),
    )
    expect(errorHtml).toContain("Network unreachable")
    expect(errorHtml).not.toMatch(/<button[^>]*disabled/)
  })

  it("falls back to generic error text when rejection is not an Error instance", async () => {
    const states: TunnelUnlockState[] = []
    const loginTunnel = vi.fn(async () => Promise.reject("boom"))

    await runTunnelUnlock(loginTunnel, "fixture-password", {
      onStateChange: (state) => {
        states.push(state)
      },
      onSuccess: () => {},
    })

    expect(states).toEqual([
      { status: "submitting" },
      { status: "error", message: "Tunnel login failed" },
    ])
    const finalState = states[1]
    if (finalState === undefined) {
      throw new Error("Expected fallback error state")
    }
    expect(isTunnelUnlockDisabled(finalState, "fixture-password")).toBe(false)
  })
})
