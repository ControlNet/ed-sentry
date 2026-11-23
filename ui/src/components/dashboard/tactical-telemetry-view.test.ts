import { describe, expect, it } from "vitest"
import type { AppSnapshot } from "@/adapters/dashboard"
import { webInterfaceHref } from "./tactical-telemetry-view"

function webStatus(overrides: Partial<AppSnapshot["web"]> = {}): AppSnapshot["web"] {
  return {
    kind: "running",
    status_label: "Running",
    bind_address: "127.0.0.1:8765",
    url: "http://127.0.0.1:8765",
    message: null,
    checked_at: "2026-06-28T12:00:00Z",
    checked_at_display: "2026-06-28 12:00:00 UTC",
    ...overrides,
  }
}

describe("webInterfaceHref", () => {
  it("rewrites IPv4 and IPv6 wildcard URLs to localhost", () => {
    expect(webInterfaceHref(webStatus({ url: "http://0.0.0.0:8765" }))).toBe(
      "http://localhost:8765",
    )
    expect(webInterfaceHref(webStatus({ url: "http://[::]:8765" }))).toBe("http://localhost:8765")
    expect(webInterfaceHref(webStatus({ url: "http://127.0.0.1:8765" }))).toBe(
      "http://127.0.0.1:8765",
    )
  })

  it("returns null when Web UI has an error message or no URL", () => {
    expect(
      webInterfaceHref(webStatus({ url: "http://[::]:8765", message: "Bind failed" })),
    ).toBeNull()
    expect(webInterfaceHref(webStatus({ url: null }))).toBeNull()
  })
})
