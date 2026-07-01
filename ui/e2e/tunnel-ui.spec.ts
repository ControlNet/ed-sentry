import { expect, test } from "@playwright/test"

test.describe.configure({ mode: "serial" })

test("@tunnel-service renders running link and QR on hover and keyboard focus", async ({
  page,
}) => {
  await page.goto("/?mock_state=tunnel_running")

  const tunnel = page.locator("[data-service-node='Tunnel']")
  const tunnelLink = tunnel.getByRole("link", { name: "task-9.trycloudflare.com" })
  const tunnelQr = page.getByTestId("tunnel-link-qr")

  await expect(tunnel).toContainText("RUNNING")
  await expect(tunnelLink).toBeVisible()
  await expect(tunnelLink).toHaveAttribute("href", "https://task-9.trycloudflare.com/")
  await expect(tunnelLink).toHaveAttribute("target", "_blank")
  await expect(tunnelLink).toHaveAttribute("rel", "noopener noreferrer")
  await expect(tunnelQr).toBeHidden()

  await tunnelLink.hover()
  await expect(tunnelQr).toBeVisible()

  await tunnelLink.focus()
  await expect(tunnelLink).toBeFocused()
  await expect(tunnelQr).toBeVisible()
})

test("@tunnel-service starts only from start or retryable error states", async ({ page }) => {
  await page.goto("/?mock_state=tunnel_start")
  const startTunnel = page.locator("[data-service-node='Tunnel']").getByRole("button", {
    name: "START",
  })

  await expect(startTunnel).toBeVisible()
  await startTunnel.click()
  await expect(page.locator("[data-service-node='Tunnel']")).toContainText("RUNNING")

  await page.goto("/?mock_state=tunnel_disabled")
  const disabledTunnel = page.locator("[data-service-node='Tunnel']")
  await expect(disabledTunnel).toContainText("UNAVAILABLE")
  await expect(disabledTunnel.getByRole("button", { name: "START" })).toHaveCount(0)

  await page.goto("/?mock_state=tunnel_running")
  const runningTunnel = page.locator("[data-service-node='Tunnel']")
  await expect(runningTunnel).toContainText("RUNNING")
  await expect(runningTunnel.getByRole("button", { name: "START" })).toHaveCount(0)
})

test("@tunnel-service starts only from retryable tunnel errors", async ({ page }) => {
  await page.goto("/?mock_state=tunnel_retryable_error")
  const retryableTunnel = page.locator("[data-service-node='Tunnel']")
  await expect(retryableTunnel).toContainText("ERROR")
  await expect(retryableTunnel).toContainText(
    "Fixture tunnel process stopped before a URL was assigned",
  )
  const retryTunnel = retryableTunnel.getByRole("button", { name: "ERROR" })
  await expect(retryTunnel).toBeVisible()
  await retryTunnel.click()
  await expect(retryableTunnel).toContainText("RUNNING")

  await page.goto("/?mock_state=tunnel_non_retryable_error")
  const nonRetryableTunnel = page.locator("[data-service-node='Tunnel']")
  await expect(nonRetryableTunnel).toContainText("ERROR")
  await expect(nonRetryableTunnel).toContainText("Fixture tunnel provider is unavailable")
  await expect(nonRetryableTunnel.getByRole("button", { name: "ERROR" })).toHaveCount(0)
})

test("@tunnel-config auth prompt blocks Systems config until login succeeds", async ({ page }) => {
  await page.goto("/?mock_state=tunnel_auth_required")
  await page.getByRole("button", { name: /Systems/u }).click()

  await expect(page.getByRole("region", { name: "Tunnel config authentication" })).toBeVisible()
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toHaveCount(0)

  await page.getByLabel("Tunnel config password").fill("fixture tunnel password")
  await page.getByRole("button", { name: "Unlock Systems" }).click()

  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toBeVisible()
  await expect(page.getByRole("region", { name: "Tunnel settings" })).toBeVisible()
})

test("@tunnel-config exposes provider, auto-start, and password controls", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /Systems/u }).click()

  const tunnelSettings = page.getByRole("region", { name: "Tunnel settings" })
  await expect(tunnelSettings.getByRole("combobox", { name: "Provider" })).toHaveValue(
    "cloudflare_quick",
  )
  await expect(tunnelSettings.getByLabel("Auto start tunnel")).not.toBeChecked()
  await expect(tunnelSettings.getByLabel("Replace tunnel config password")).toHaveValue("")
  await expect(tunnelSettings.getByLabel("Clear tunnel config password on save")).not.toBeChecked()
})
