import { expect, type Locator, type Page, test } from "@playwright/test"

const evidenceDir = "../.omo/evidence/gui-webui-tauri"
const forbiddenScreenshotText = [
  "TEST_FIXTURE_MATRIX_TOKEN_DO_NOT_USE_2035",
  "fixture-smoke-access-token",
  "private-journal-root",
] as const

test("@responsive-mobile captures the 375px WebUI state without horizontal overflow", async ({
  page,
}) => {
  await captureResponsiveDashboard(page, {
    width: 375,
    path: `${evidenceDir}/task-12-mobile.png`,
    label: "mobile",
  })
})

test("@responsive-tablet captures the 768px WebUI state without horizontal overflow", async ({
  page,
}) => {
  await captureResponsiveDashboard(page, {
    width: 768,
    path: `${evidenceDir}/task-12-tablet.png`,
    label: "tablet",
  })
})

test("@responsive-desktop captures the 1280px WebUI state without horizontal overflow", async ({
  page,
}) => {
  await captureResponsiveDashboard(page, {
    width: 1280,
    path: `${evidenceDir}/task-12-desktop.png`,
    label: "desktop",
  })
})

test("@keyboard-focus reaches core controls and shows focus affordance", async ({ page }) => {
  await page.goto("/")

  const dashboardButton = page.getByRole("button", { name: "Dashboard" })
  const configButton = page.getByRole("button", { name: "Config" })

  await pressTabUntilFocused(page, dashboardButton)
  await expectVisibleFocus(dashboardButton, "Dashboard")

  await pressTabUntilFocused(page, configButton)
  await expectVisibleFocus(configButton, "Config")
  await page.keyboard.press("Enter")
  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()

  const journalFolder = page.getByRole("textbox", { name: "Journal folder" })
  await pressTabUntilFocused(page, journalFolder)
  await expectVisibleFocus(journalFolder, "Journal folder")
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A")
  await page.keyboard.type("Sanitized Journal source QA")

  const saveButton = page.getByRole("button", { name: "Save" })
  const cancelButton = page.getByRole("button", { name: "Cancel" })
  await expect(saveButton).toBeEnabled()
  await expect(cancelButton).toBeEnabled()

  await pressTabUntilFocused(page, saveButton, { reverse: true })
  await expectVisibleFocus(saveButton, "Save")

  await pressTabUntilFocused(page, cancelButton, { reverse: true })
  await expectVisibleFocus(cancelButton, "Cancel")

  await page.screenshot({
    path: `${evidenceDir}/task-12-keyboard-focus.png`,
    fullPage: true,
  })
})

test("@reduced-motion keeps controls usable while disabling non-essential motion", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" })
  await page.goto("/")

  const refreshButton = page.getByRole("button", { name: "Refresh" })
  await expect(refreshButton).toBeVisible()
  await expect(page.getByRole("region", { name: "Connection state" })).toContainText("Mock live")

  const transitionDurations = await refreshButton.evaluate(
    (element) => getComputedStyle(element).transitionDuration,
  )
  expect(parseCssTimeListMs(transitionDurations).every((duration) => duration === 0)).toBe(true)

  await page.goto("/?mock_state=loading")
  await expect(page.getByText("Loading dashboard snapshot")).toBeVisible()
  const animationDurations = await page
    .locator(".animate-spin")
    .first()
    .evaluate((element) => getComputedStyle(element).animationDuration)
  expect(parseCssTimeListMs(animationDurations).every((duration) => duration <= 1)).toBe(true)
  console.log(
    `REDUCED_MOTION_OBSERVABLE: transitionDuration=${transitionDurations} animationDuration=${animationDurations}`,
  )

  await page.screenshot({
    path: `${evidenceDir}/task-12-reduced-motion.png`,
    fullPage: true,
  })
})

test("@state-coverage renders empty, loading, and error dashboard states", async ({ page }) => {
  await page.goto("/?mock_state=empty")
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "No dashboard events have arrived.",
  )
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "No tracked missions in this snapshot.",
  )
  await assertNoForbiddenText(page)
  await page.screenshot({
    path: `${evidenceDir}/task-12-empty-state.png`,
    fullPage: true,
  })

  await page.goto("/?mock_state=loading")
  await expect(page.getByText("Loading dashboard snapshot")).toBeVisible()
  await page.screenshot({
    path: `${evidenceDir}/task-12-loading-state.png`,
    fullPage: true,
  })

  await page.goto("/?mock_state=error")
  await expect(page.getByRole("heading", { name: "Dashboard unavailable" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Retry" })).toBeVisible()
  await assertNoForbiddenText(page)
  await page.screenshot({
    path: `${evidenceDir}/task-12-error-state.png`,
    fullPage: true,
  })
})

test("@accessibility-smoke exposes landmarks, region names, and form labels", async ({ page }) => {
  await page.goto("/")

  await expect(page.locator("main")).toBeVisible()
  await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible()
  await expect(page.getByRole("heading", { name: "AFK monitor" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Connection state" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Combat metrics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Refresh" })).toBeEnabled()

  await page.getByRole("button", { name: "Config" }).click()
  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Journal settings" })).toBeVisible()
  await expect(page.getByRole("textbox", { name: "Journal folder" })).toBeVisible()
  await expect(page.getByRole("spinbutton", { name: "Recent files" })).toBeVisible()
  await expect(page.getByRole("textbox", { name: "Host" })).toBeVisible()
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toBeVisible()
  await expect(page.getByLabel("Replace access token")).toHaveValue("")
  await assertNoForbiddenText(page)

  console.log(
    "A11Y_OBSERVABLE: main landmark, Primary nav, named dashboard regions, config form labels, and write-only token input are reachable",
  )
})

type ResponsiveCapture = {
  readonly width: number
  readonly path: string
  readonly label: string
}

async function captureResponsiveDashboard(
  page: Page,
  { width, path, label }: ResponsiveCapture,
): Promise<void> {
  await page.setViewportSize({ width, height: 900 })
  await page.goto("/")
  await expect(page.getByRole("heading", { name: "AFK monitor" })).toBeVisible()
  await expect(page.locator("header")).toContainText("Local Commander")
  await expect(page.getByRole("region", { name: "Connection state" })).toContainText("Mock live")
  await expect(page.getByRole("region", { name: "Combat metrics" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
    "Bounty voucher",
  )
  await assertNoHorizontalOverflow(page, label)
  await assertNoForbiddenText(page)
  await page.screenshot({ path, fullPage: true })
  console.log(`SCREENSHOT: ${path}`)
}

async function assertNoHorizontalOverflow(page: Page, label: string): Promise<void> {
  const viewport = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
    bodyScrollWidth: document.body.scrollWidth,
  }))
  console.log(
    `RESPONSIVE_OBSERVABLE: ${label} clientWidth=${viewport.clientWidth} scrollWidth=${viewport.scrollWidth} bodyScrollWidth=${viewport.bodyScrollWidth}`,
  )
  expect(viewport.scrollWidth).toBeLessThanOrEqual(viewport.clientWidth)
  expect(viewport.bodyScrollWidth).toBeLessThanOrEqual(viewport.clientWidth)
}

async function assertNoForbiddenText(page: Page): Promise<void> {
  const bodyText = (await page.locator("body").textContent()) ?? ""
  for (const forbiddenText of forbiddenScreenshotText) {
    expect(bodyText).not.toContain(forbiddenText)
  }
}

async function pressTabUntilFocused(
  page: Page,
  target: Locator,
  options: { readonly reverse?: boolean } = {},
): Promise<void> {
  const key = options.reverse === true ? "Shift+Tab" : "Tab"
  for (let index = 0; index < 40; index += 1) {
    await page.keyboard.press(key)
    if (await target.evaluate((element) => element === document.activeElement)) {
      return
    }
  }
  throw new Error(`Tab navigation did not reach ${await targetDescription(target)}`)
}

async function expectVisibleFocus(target: Locator, label: string): Promise<void> {
  await expect(target).toBeFocused()
  const focusStyle = await target.evaluate((element) => {
    const style = getComputedStyle(element)
    return {
      outlineStyle: style.outlineStyle,
      boxShadow: style.boxShadow,
    }
  })
  console.log(
    `FOCUS_OBSERVABLE: ${label} outlineStyle=${focusStyle.outlineStyle} boxShadow=${focusStyle.boxShadow}`,
  )
  expect(focusStyle.outlineStyle !== "none" || focusStyle.boxShadow !== "none").toBe(true)
}

async function targetDescription(target: Locator): Promise<string> {
  const text = await target.textContent()
  return text?.replaceAll(/\s+/g, " ").trim() ?? "target"
}

function parseCssTimeListMs(value: string): number[] {
  return value.split(",").map((item) => parseCssTimeMs(item.trim()))
}

function parseCssTimeMs(value: string): number {
  if (value.endsWith("ms")) {
    return Number.parseFloat(value)
  }
  if (value.endsWith("s")) {
    return Number.parseFloat(value) * 1000
  }
  return Number.NaN
}
