import type { Page } from "@playwright/test"
import { expect, test } from "@playwright/test"

test("dashboard scaffold renders monitor state when mock adapter is active", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByRole("button", { name: /Dashboard/u })).toBeVisible()
  await expect(page.getByRole("heading", { name: "Telemetry Interface" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Telemetry Link" })).toContainText(
    "Local Commander",
  )
  await expect(page.getByRole("region", { name: "Service Nodes" })).toContainText(
    "No production API configured",
  )

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-4-dashboard-smoke.png")
})

test("@webui-assets serves the configured favicon", async ({ page }) => {
  await page.goto("/")

  await expect(page.locator("link[rel='icon']")).toHaveAttribute("href", "/favicon.ico")

  const response = await page.request.get("/favicon.ico")
  expect(response.ok()).toBe(true)
  expect(response.headers()["content-type"]).toContain("image/")
  expect((await response.body()).length).toBeGreaterThan(1_000)
})

test("@mock-dashboard renders the adapter-backed dashboard shell", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible()
  await expect(page.getByRole("heading", { name: "Telemetry Interface" })).toBeVisible()
  await expect(page.locator("main")).toContainText("SYS_RELAY: CONNECTED")
  await expect(page.getByRole("region", { name: "Combat Analytics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Bounty voucher",
  )
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByTestId("telemetry-active-mission-count")).toHaveText("2")
  await expect(page.getByRole("region", { name: "Mission progress" })).not.toContainText("TOTAL")
  await expect(page.getByRole("region", { name: "Mission progress" })).not.toContainText(
    "1/2 active",
  )
  await expect(page.getByRole("region", { name: "Service Nodes" })).toContainText(
    "Sanitized Journal source",
  )

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.png")

  await page.setViewportSize({ width: 768, height: 1024 })
  await page.goto("/")
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-9-dashboard-tablet.png")

  await page.setViewportSize({ width: 375, height: 900 })
  await page.goto("/")
  await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible()
  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-9-dashboard-mobile.png")
})

test("@todo10-dashboard renders all first-milestone operational regions", async ({ page }) => {
  const consoleErrors: string[] = []
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text())
    }
  })

  await page.goto("/")

  const checklist = page.getByRole("region", { name: "Checklist" })

  await expect(page.locator("main")).toContainText("Local Commander")
  await expect(page.getByRole("heading", { name: "Telemetry Interface" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Combat Analytics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Ship Integrity" })).toHaveCount(0)
  await expect(checklist).toBeVisible()
  const checklistRows = checklist.getByTestId("afk-checklist-row")
  const expectedChecklistRows = [
    ["Hardpoints deployed", "PASS"],
    ["Engine pips zero", "PASS"],
    ["Cargo loaded", "FAIL"],
  ] as const
  await expect(checklistRows).toHaveCount(expectedChecklistRows.length)
  for (const [index, [label, stateLabel]] of expectedChecklistRows.entries()) {
    await expect(checklistRows.nth(index)).toContainText(label)
    await expect(checklistRows.nth(index)).toContainText(stateLabel)
  }
  await expect(checklist).not.toContainText("Status Flags")
  await expect(checklist).not.toContainText("Status.json")
  await expect(checklist).not.toContainText("Cargo.json")
  await expect(page.getByRole("region", { name: "Service Nodes" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Bounty voucher",
  )
  await expect(page.getByRole("region", { name: "Recent event feed" })).not.toContainText(
    "runtime_status",
  )
  await expect(page.getByRole("region", { name: "Recent event feed" })).not.toContainText(
    /[💥⌚🎯]/u,
  )
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByRole("region", { name: "Service Nodes" })).toContainText("Disabled")
  await expect(page.getByRole("region", { name: "Service Nodes" })).toContainText("Mock live")

  await checklist.scrollIntoViewIfNeeded()
  await checklist.screenshot({
    path: "../.omo/evidence/afk-checklist-watcher/task-10-checklist-panel.png",
  })
  expect(consoleErrors).toEqual([])
})

test("@event-feed keeps long history bounded and newest first", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto("/?mock_state=long_feed")

  const feed = page.getByRole("region", { name: "Recent event feed" })
  const rows = feed.getByTestId("telemetry-event-row")
  await expect(rows.first()).toContainText("Long feed event 60")
  await expect(rows).toHaveCount(30)
  await expect(rows.last()).toContainText("Long feed event 31")

  const feedBox = await feed.boundingBox()
  if (feedBox === null) {
    throw new Error("Recent event feed did not render a measurable box")
  }
  expect(feedBox.height).toBeLessThanOrEqual(620)

  const listMetrics = await feed.locator(".custom-scrollbar").evaluate((list) => ({
    clientHeight: list.clientHeight,
    scrollHeight: list.scrollHeight,
  }))
  expect(listMetrics.scrollHeight).toBeGreaterThan(listMetrics.clientHeight)

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/event-feed-long-bounded.png")
})

test("@service-nodes shows absolute Journal source details in service nodes", async ({ page }) => {
  await page.goto("/?mock_state=private_path")

  const serviceNodes = page.getByRole("region", { name: "Service Nodes" })
  await expect(serviceNodes).toContainText(
    "/home/private-journal-root/Elite Dangerous/Journal.private.2036-01-02.log",
  )
  await expect(serviceNodes).not.toContainText("Default journal folder")
  await expect(serviceNodes).not.toContainText("Selected Journal file")
})

test("@service-nodes expose unified semantic status encoding", async ({ page }) => {
  await page.goto("/?mock_state=service_statuses")

  const serviceNodes = page.getByRole("region", { name: "Service Nodes" })
  await expect(serviceNodes.locator("[data-service-node='Local Journal']")).toHaveAttribute(
    "data-status-kind",
    "running",
  )
  await expect(serviceNodes.locator("[data-service-node='Matrix Relay']")).toHaveAttribute(
    "data-status-kind",
    "running",
  )
  await expect(serviceNodes.locator("[data-service-node='Matrix Relay']")).toContainText(
    "#ed-sentry:example.org",
  )
  await expect(serviceNodes.locator("[data-service-node='Web Interface']")).toHaveAttribute(
    "data-status-kind",
    "disabled",
  )
})

test("@service-nodes renders Web Interface URL as a clickable link", async ({ page }) => {
  await page.goto("/?mock_state=web_url")

  const webInterface = page.locator("[data-service-node='Web Interface']")
  const webLink = webInterface.getByRole("link", { name: "0.0.0.0:8765" })

  await expect(webLink).toBeVisible()
  await expect(webLink).toHaveAttribute("href", "http://localhost:8765")
  await expect(webLink).toHaveAttribute("target", "_blank")
  await expect(webLink).toHaveAttribute("rel", "noopener noreferrer")

  await page.goto("/?mock_state=web_lan_url")
  const lanLink = page
    .locator("[data-service-node='Web Interface']")
    .getByRole("link", { name: "192.168.50.10:8765" })

  await expect(lanLink).toBeVisible()
  await expect(lanLink).toHaveAttribute("href", "http://192.168.50.10:8765")
  await expect(lanLink).toHaveAttribute("target", "_blank")
  await expect(lanLink).toHaveAttribute("rel", "noopener noreferrer")
})

test("@about workspace renders project information and compliance links", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()

  await expect(page.getByRole("heading", { name: "About Interface" })).toBeVisible()
  await expect(page.locator("main").getByRole("heading", { name: "ed-sentry" })).toBeVisible()
  await expect(page.locator("body")).toContainText("Elite Dangerous AFK Sentry")
  await expect(page.getByRole("region", { name: "System Information" })).toContainText(
    /0\.1\.0-\d{8}/u,
  )
  await expect(page.getByRole("region", { name: "System Information" })).toContainText(
    "GNU Affero General Public License v3.0",
  )
  await expect(page.getByRole("link", { name: "CMDR ControlNet" })).toHaveAttribute(
    "href",
    "https://inara.cz/elite/cmdr/78197/",
  )
  await expect(page.getByRole("link", { name: "CMDR ControlNet" })).toHaveAttribute(
    "target",
    "_blank",
  )
  await expect(page.getByRole("link", { name: /Source Repository/u })).toHaveAttribute(
    "href",
    "https://github.com/ControlNet/ed-sentry",
  )
  await expect(page.getByRole("link", { name: /Source Repository/u })).toHaveAttribute(
    "target",
    "_blank",
  )
  await expect(page.getByRole("region", { name: "Legal & Compliance" })).toContainText(
    "Unofficial Software",
  )

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/about-workspace.png")
})

test("@about workspace remains usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()

  await expect(page.getByRole("heading", { name: "About Interface" })).toBeVisible()
  await expect(page.getByRole("region", { name: "System Information" })).toBeVisible()
  await expect(page.locator("body")).toContainText("Elite Dangerous AFK Sentry")
})

test("@header status indicator shrinks to content width", async ({ page }) => {
  await page.goto("/")

  const statusWidth = await page
    .locator("[data-titlebar-drag-region='status']")
    .evaluate((element) => element.getBoundingClientRect().width)
  expect(statusWidth).toBeLessThanOrEqual(96)
})

test("@about workspace preserves telemetry navigation", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()
  await page.getByRole("button", { name: /Dashboard/u }).click()

  await expect(page.getByRole("heading", { name: "Telemetry Interface" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Service Nodes" })).toBeVisible()
})

test("@connection-status reflects degraded live transport state", async ({ page }) => {
  await page.goto("/?mock_state=degraded_connection")

  await expect(page.locator("header")).toContainText("DEGRADED")
  await expect(page.locator("header")).not.toContainText("SYNCED")
})

test("@loading-screen renders the tactical startup visual while awaiting a snapshot", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto("/?mock_state=loading")

  await expect(page.getByRole("region", { name: "Dashboard startup" })).toBeVisible()
  await expectLoadingProgress(page)
  await expect(page.locator("svg").first()).toBeVisible()
  await expect(page.getByText("Loading dashboard snapshot")).toBeVisible()

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/loading-screen.png")

  await page.setViewportSize({ width: 768, height: 1024 })
  await page.goto("/?mock_state=loading")
  await expect(page.getByRole("region", { name: "Dashboard startup" })).toBeVisible()
  await expectLoadingProgress(page)
  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/loading-screen-tablet.png")

  await page.setViewportSize({ width: 375, height: 812 })
  await page.goto("/?mock_state=loading")
  await expect(page.getByRole("region", { name: "Dashboard startup" })).toBeVisible()
  await expectLoadingProgress(page)
  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/loading-screen-mobile.png")
})

async function expectLoadingProgress(page: Page): Promise<void> {
  await expect(page.getByText(/PARSING FLIGHT JOURNAL|ESTABLISHING MATRIX RELAY/u)).toBeVisible({
    timeout: 5_000,
  })
}

test("@missions workspace fits a short desktop viewport", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 640 })
  await page.goto("/")
  await page.getByRole("button", { name: /Missions/u }).click()

  await expect(page.getByRole("heading", { name: "Missions Interface" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Mission Directory" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Mission Intel" })).toBeVisible()

  const mainMetrics = await page.locator("main").evaluate((main) => ({
    clientHeight: main.clientHeight,
    scrollHeight: main.scrollHeight,
  }))
  expect(mainMetrics.scrollHeight).toBeLessThanOrEqual(mainMetrics.clientHeight + 4)

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/missions-short-viewport.png")
})

test("@config-edit saves a non-secret setting and reloads the config view", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /Systems/u }).click()
  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()

  const port = page.getByRole("spinbutton", { name: "Port", exact: true })
  await expect(port).toHaveValue("4173")
  await port.fill("4273")
  await expect(page.getByText("Autosave pending")).toBeVisible()
  await expect(page.getByText("All changes saved")).toBeVisible()

  await page.getByRole("button", { name: "Dashboard" }).click()
  await page.getByRole("button", { name: /Systems/u }).click()
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toHaveValue("4273")

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-11-config-edit.png")
})

test("@config-edit allows clearing Journal folder to use the default", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /Systems/u }).click()

  const journalFolder = page.getByRole("textbox", { name: "Journal folder" })
  await journalFolder.fill("")
  await expect(page.getByText("Journal folder is required.")).toHaveCount(0)
  await expect(page.getByText("Autosave pending")).toBeVisible()
  await expect(page.getByText("All changes saved")).toBeVisible()

  await page.getByRole("button", { name: "Dashboard" }).click()
  await page.getByRole("button", { name: /Systems/u }).click()
  await expect(page.getByRole("textbox", { name: "Journal folder" })).toHaveValue("")
})

test("@token-mask does not render the stored Matrix token fixture", async ({ page }) => {
  const fixtureToken = "TEST_FIXTURE_MATRIX_TOKEN_DO_NOT_USE_2035"
  await page.goto("/")
  await page.getByRole("button", { name: /Systems/u }).click()

  await expect(page.getByText("TOKEN PRESENT IN VAULT")).toBeVisible()
  await expect(page.getByLabel("Replace access token")).toHaveValue("")
  await expect(page.getByText(fixtureToken)).toHaveCount(0)
  const textDump = await page.locator("body").textContent()
  const htmlDump = await page.locator("body").evaluate((body) => body.innerHTML)
  expect(textDump ?? "").not.toContain(fixtureToken)
  expect(htmlDump).not.toContain(fixtureToken)

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/task-11-token-mask.png")
})

async function captureFullPage(page: Page, path: string): Promise<void> {
  await page.screenshot({ path, fullPage: true })
}
