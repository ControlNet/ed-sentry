import type { Page } from "@playwright/test"
import { expect, test } from "@playwright/test"

test("dashboard scaffold renders monitor state when mock adapter is active", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByRole("button", { name: /Dashboard/u })).toBeVisible()
  await expect(page.getByRole("heading", { name: "Dashboard", level: 1 })).toBeVisible()
  await expect(page.locator("footer")).toContainText("CMDR Local Commander")
  await expect(page.getByRole("region", { name: "Services" })).toContainText(
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
  await expect(page.getByRole("heading", { name: "Dashboard", level: 1 })).toBeVisible()
  await expect(page.locator("header [data-connection-status='connected']")).toContainText("Synced")
  await expect(page.getByRole("region", { name: "Combat" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent events" })).toContainText("Bounty voucher")
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByTestId("telemetry-active-mission-count")).toHaveText("2")
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "12 kills left",
  )
  await expect(page.getByRole("region", { name: "Services" })).toContainText(
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

  const shipStatus = page.getByRole("region", { name: "Ship status and AFK readiness" })

  await expect(page.locator("footer")).toContainText("Local Commander")
  await expect(page.getByRole("heading", { name: "Dashboard", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Combat" })).toBeVisible()
  await expect(shipStatus).toBeVisible()
  await expect(shipStatus).toContainText("92%")
  await expect(page.getByTestId("afk-readiness-count")).toHaveText("2 / 3 pass")
  const checklistRows = shipStatus.getByTestId("afk-checklist-row")
  const expectedChecklistRows = [
    ["Hardpoints deployed", "Pass", "Status.json"],
    ["Engine pips zero", "Pass", "Status.json"],
    ["Cargo loaded", "Fail", "Cargo.json"],
  ] as const
  await expect(checklistRows).toHaveCount(expectedChecklistRows.length)
  for (const [index, [label, stateLabel, source]] of expectedChecklistRows.entries()) {
    await expect(checklistRows.nth(index)).toContainText(label)
    await expect(checklistRows.nth(index)).toContainText(stateLabel)
    await expect(checklistRows.nth(index)).toContainText(source)
  }
  await expect(shipStatus).not.toContainText("Status Flags")
  await expect(page.getByRole("region", { name: "Services" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent events" })).toContainText("Bounty voucher")
  await expect(page.getByRole("region", { name: "Recent events" })).not.toContainText(
    "runtime_status",
  )
  await expect(page.getByRole("region", { name: "Recent events" })).not.toContainText(/[💥⌚🎯]/u)
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByRole("region", { name: "Services" })).toContainText("Disabled")
  await expect(page.getByRole("region", { name: "Services" })).toContainText("Mock live")

  await shipStatus.scrollIntoViewIfNeeded()
  await shipStatus.screenshot({
    path: "../.omo/evidence/afk-checklist-watcher/task-10-checklist-panel.png",
  })
  expect(consoleErrors).toEqual([])
})

test("@event-feed keeps long history bounded and newest first", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto("/?mock_state=long_feed")

  const recent = page.getByRole("region", { name: "Recent events" })
  const recentRows = recent.getByTestId("telemetry-event-row")
  await expect(recentRows).toHaveCount(5)
  await expect(recentRows.first()).toContainText("Long feed event 60")
  await expect(recentRows.last()).toContainText("Long feed event 56")

  await recent.getByRole("button", { name: "All events" }).click()
  await expect(page.getByRole("heading", { name: "Events", level: 1 })).toBeVisible()
  const log = page.getByRole("region", { name: "Event log" })
  const rows = log.getByTestId("event-log-entry")
  await expect(rows.first()).toContainText("Long feed event 60")
  await expect(rows).toHaveCount(30)
  await expect(rows.last()).toContainText("Long feed event 31")
  await expect(log).toContainText("30 shown")

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/event-feed-long-bounded.png")
})

test("@events filters the log by event category", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: "Events", exact: true }).click()

  const timeline = page.getByRole("region", { name: "Session timeline" })
  const log = page.getByRole("region", { name: "Event log" })
  await expect(timeline.getByRole("button", { name: /All\s*4/u })).toHaveAttribute(
    "aria-pressed",
    "true",
  )
  await expect(log.getByTestId("event-log-entry")).toHaveCount(4)

  await timeline.getByRole("button", { name: /Kill\s*1/u }).click()
  await expect(log.getByTestId("event-log-entry")).toHaveCount(1)
  await expect(log).toContainText("Bounty voucher recorded")
  await expect(log).toContainText("1 shown")
})

test("@service-nodes shows absolute Journal source details in service nodes", async ({ page }) => {
  await page.goto("/?mock_state=private_path")

  const serviceNodes = page.getByRole("region", { name: "Services" })
  await expect(serviceNodes).toContainText(
    "/home/private-journal-root/Elite Dangerous/Journal.private.2036-01-02.log",
  )
  await expect(serviceNodes).not.toContainText("Default journal folder")
  await expect(serviceNodes).not.toContainText("Selected Journal file")
})

test("@service-nodes expose unified semantic status encoding", async ({ page }) => {
  await page.goto("/?mock_state=service_statuses")

  const serviceNodes = page.getByRole("region", { name: "Services" })
  await expect(serviceNodes.locator("[data-service-node='Journal']")).toHaveAttribute(
    "data-status-kind",
    "running",
  )
  await expect(serviceNodes.locator("[data-service-node='Matrix']")).toHaveAttribute(
    "data-status-kind",
    "running",
  )
  await expect(serviceNodes.locator("[data-service-node='Matrix']")).toContainText(
    "#ed-sentry:example.org",
  )
  await expect(serviceNodes.locator("[data-service-node='Web UI']")).toHaveAttribute(
    "data-status-kind",
    "disabled",
  )
})

test("@service-nodes renders Web UI URL as a clickable link", async ({ page }) => {
  await page.goto("/?mock_state=web_url")

  const webInterface = page.locator("[data-service-node='Web UI']")
  const webLink = webInterface.getByRole("link", { name: "0.0.0.0:8765" })

  await expect(webLink).toBeVisible()
  await expect(webLink).toHaveAttribute("href", "http://localhost:8765")
  await expect(webLink).toHaveAttribute("target", "_blank")
  await expect(webLink).toHaveAttribute("rel", "noopener noreferrer")

  await page.goto("/?mock_state=web_lan_url")
  const lanLink = page
    .locator("[data-service-node='Web UI']")
    .getByRole("link", { name: "192.168.50.10:8765" })

  await expect(lanLink).toBeVisible()
  await expect(lanLink).toHaveAttribute("href", "http://192.168.50.10:8765")
  await expect(lanLink).toHaveAttribute("target", "_blank")
  await expect(lanLink).toHaveAttribute("rel", "noopener noreferrer")
})

test("@about workspace renders project information and compliance links", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()

  await expect(page.getByRole("heading", { name: "About", level: 1 })).toBeVisible()
  const info = page.getByRole("article", { name: "System information" })
  await expect(info).toContainText("ED Sentry")
  await expect(info).toContainText("Elite Dangerous AFK Sentry")
  await expect(page.getByTestId("about-version")).toHaveText(/^\d+\.\d+\.\d+-\d{8}$/u)
  await expect(info).toContainText("GNU Affero General Public License v3.0")
  await expect(page.getByRole("link", { name: "CMDR ControlNet" })).toHaveAttribute(
    "href",
    "https://inara.cz/elite/cmdr/78197/",
  )
  await expect(page.getByRole("link", { name: "CMDR ControlNet" })).toHaveAttribute(
    "target",
    "_blank",
  )
  const repository = page.getByRole("link", { name: "github.com/ControlNet/ed-sentry" })
  await expect(repository).toHaveAttribute("href", "https://github.com/ControlNet/ed-sentry")
  await expect(repository).toHaveAttribute("target", "_blank")
  await expect(page.getByRole("region", { name: "Legal and compliance" })).toContainText(
    "Unofficial software",
  )
  await expect(page.getByRole("region", { name: "Data sources" })).toContainText("Status.json")

  await captureFullPage(page, "../.omo/evidence/gui-webui-tauri/about-workspace.png")
})

test("@about workspace remains usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()

  await expect(page.getByRole("heading", { name: "About", level: 1 })).toBeVisible()
  await expect(page.getByRole("article", { name: "System information" })).toBeVisible()
  await expect(page.locator("body")).toContainText("Elite Dangerous AFK Sentry")
})

test("@header status indicator shrinks to content width", async ({ page }) => {
  await page.goto("/")

  const statusWidth = await page
    .locator("[data-titlebar-drag-region='status-label']")
    .evaluate((element) => element.getBoundingClientRect().width)
  expect(statusWidth).toBeLessThanOrEqual(110)
})

test("@about workspace preserves telemetry navigation", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: /About/u }).click()
  await page.getByRole("button", { name: /Dashboard/u }).click()

  await expect(page.getByRole("heading", { name: "Dashboard", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Services" })).toBeVisible()
})

test("@connection-status reflects degraded live transport state", async ({ page }) => {
  await page.goto("/?mock_state=degraded_connection")

  await expect(page.locator("header")).toContainText("Degraded")
  await expect(page.locator("header")).not.toContainText("Synced")
})

test("@loading-screen shows the adapter detail and an indeterminate bar", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto("/?mock_state=loading")

  await expect(page.getByRole("region", { name: "Dashboard startup" })).toBeVisible()
  await expectLoadingProgress(page)
  await expect(page.getByText("Loading dashboard snapshot")).toBeVisible()
  await expect(page.getByText("%", { exact: true })).toHaveCount(0)

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
  await expect(page.getByRole("heading", { name: "Waiting for first snapshot" })).toBeVisible()
  await expect(page.locator(".ld-run")).toBeVisible()
}

test("@missions workspace scrolls inside the content pane on a short desktop viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 640 })
  await page.goto("/")
  await page.getByRole("button", { name: /Missions/u }).click()

  await expect(page.getByRole("heading", { name: "Missions", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Active missions" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Mission details" })).toBeVisible()

  const pageMetrics = await page.evaluate(() => ({
    clientHeight: document.documentElement.clientHeight,
    scrollHeight: document.documentElement.scrollHeight,
  }))
  expect(pageMetrics.scrollHeight).toBeLessThanOrEqual(pageMetrics.clientHeight)
  await expect(page.locator("header")).toBeInViewport()
  await expect(page.locator("footer")).toBeInViewport()
  await page.getByRole("region", { name: "Mission timeline" }).scrollIntoViewIfNeeded()
  await expect(page.getByRole("region", { name: "Mission timeline" })).toBeInViewport()

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

  await expect(page.getByTestId("matrix-token-state")).toHaveText("Access token stored")
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
