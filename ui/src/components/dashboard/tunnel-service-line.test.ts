import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"
import type { TunnelStatusView } from "@/adapters/dashboard"
import {
  runTunnelStart,
  TunnelServiceLineView,
  type TunnelStartRequestState,
} from "./tunnel-service-line"

const startableTunnel: TunnelStatusView = {
  kind: "start",
  status_label: "Start",
  provider: "cloudflare_quick",
  provider_label: "Cloudflare Quick Tunnel",
  session_id: null,
  message: null,
  public_url: null,
  checked_at: null,
  checked_at_display: null,
  retryable_error: false,
}

describe("TunnelServiceLine start failure recovery", () => {
  it("catches a rejected start, renders the error, and re-enables the Retry button", async () => {
    const states: TunnelStartRequestState[] = []
    const onStart = vi.fn(async () => {
      throw new Error("Request timed out after 15s")
    })

    await runTunnelStart(onStart, (state) => {
      states.push(state)
    })

    expect(onStart).toHaveBeenCalledTimes(1)
    expect(states).toEqual([
      { status: "starting" },
      { status: "error", message: "Request timed out after 15s" },
    ])

    const [startingState, errorState] = states
    if (startingState === undefined || errorState === undefined) {
      throw new Error("Expected starting and error states")
    }

    const startingHtml = renderToStaticMarkup(
      createElement(TunnelServiceLineView, {
        tunnel: startableTunnel,
        startState: startingState,
        qrVisible: false,
        tunnelLinkRef: { current: null },
        onQrVisibleChange: () => {},
        onStartClick: () => {},
      }),
    )
    expect(startingHtml).toMatch(/<button[^>]*disabled=""/)

    const errorHtml = renderToStaticMarkup(
      createElement(TunnelServiceLineView, {
        tunnel: startableTunnel,
        startState: errorState,
        qrVisible: false,
        tunnelLinkRef: { current: null },
        onQrVisibleChange: () => {},
        onStartClick: () => {},
      }),
    )
    expect(errorHtml).toContain('data-status-kind="error"')
    expect(errorHtml).toContain('role="alert"')
    expect(errorHtml).toContain("Request timed out after 15s")
    expect(errorHtml).toContain(">Retry<")
    expect(errorHtml).not.toMatch(/<button[^>]*disabled/)
  })

  it("falls back to a generic error when start rejects with a non-Error value", async () => {
    const states: TunnelStartRequestState[] = []
    await runTunnelStart(
      async () => Promise.reject("boom"),
      (state) => {
        states.push(state)
      },
    )

    expect(states).toEqual([
      { status: "starting" },
      { status: "error", message: "Tunnel start failed" },
    ])
  })
})
