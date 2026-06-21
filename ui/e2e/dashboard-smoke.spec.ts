import { expect, test } from "@playwright/test"

test("dashboard scaffold renders monitor state when mock adapter is active", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByText("Local Commander in Sanitized System")).toBeVisible()
  await expect(page.getByRole("region", { name: "Combat metrics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Web status" })).toContainText(
    "No production API configured",
  )

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-4-dashboard-smoke.png",
    fullPage: true,
  })
})

test("@mock-dashboard renders the adapter-backed dashboard shell", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible()
  await expect(page.getByRole("heading", { name: "AFK monitor" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Connection state" })).toContainText("Mock live")
  await expect(page.getByRole("region", { name: "Combat metrics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Bounty voucher",
  )
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByRole("region", { name: "Journal source" })).toContainText(
    "Sanitized Journal source",
  )

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.png",
    fullPage: true,
  })

  await page.setViewportSize({ width: 768, height: 1024 })
  await page.goto("/")
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-9-dashboard-tablet.png",
    fullPage: true,
  })

  await page.setViewportSize({ width: 375, height: 900 })
  await page.goto("/")
  await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible()
  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-9-dashboard-mobile.png",
    fullPage: true,
  })
})

test("@todo10-dashboard renders all first-milestone operational regions", async ({ page }) => {
  await page.goto("/")

  await expect(page.locator("header")).toContainText("Local Commander")
  await expect(page.getByRole("region", { name: "Connection state" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Combat metrics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Health and fuel" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Warning rail" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Bounty voucher",
  )
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Kills -/h uptime 0s missions 0/0",
  )
  await expect(page.getByRole("region", { name: "Recent event feed" })).not.toContainText(
    /[💥⌚🎯]/u,
  )
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByRole("region", { name: "Journal source" })).toContainText(
    "Sanitized Journal source",
  )
  await expect(page.getByRole("region", { name: "Matrix status" })).toContainText("Disabled")
  await expect(page.getByRole("region", { name: "Web status" })).toContainText("Mock live")
})

test("@config-edit saves a non-secret setting and reloads the config view", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: "Config" }).click()
  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()

  const port = page.getByRole("spinbutton", { name: "Port", exact: true })
  await expect(port).toHaveValue("4173")
  await port.fill("4273")
  await expect(page.getByRole("button", { name: "Save" })).toBeEnabled()
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Config saved")).toBeVisible()

  await page.getByRole("button", { name: "Dashboard" }).click()
  await page.getByRole("button", { name: "Config" }).click()
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toHaveValue("4273")

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-11-config-edit.png",
    fullPage: true,
  })
})

test("@token-mask does not render the stored Matrix token fixture", async ({ page }) => {
  const fixtureToken = "TEST_FIXTURE_MATRIX_TOKEN_DO_NOT_USE_2035"
  await page.goto("/")
  await page.getByRole("button", { name: "Config" }).click()

  await expect(page.getByText("Token stored")).toBeVisible()
  await expect(page.getByLabel("Replace access token")).toHaveValue("")
  await expect(page.getByText(fixtureToken)).toHaveCount(0)
  const textDump = await page.locator("body").textContent()
  const htmlDump = await page.locator("body").evaluate((body) => body.innerHTML)
  expect(textDump ?? "").not.toContain(fixtureToken)
  expect(htmlDump).not.toContain(fixtureToken)

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/task-11-token-mask.png",
    fullPage: true,
  })
})
