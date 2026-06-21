import { appendFileSync } from "node:fs"
import { chromium, expect } from "@playwright/test"

const options = parseArgs(process.argv.slice(2))
const scenario = requireOption(options, "scenario")
const url = requireOption(options, "url")
const screenshot = options.get("screenshot")
const responsiveDir = options.get("responsive-dir")
const appendJournal = options.get("append-journal")

const browser = await chromium.launch()
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } })

try {
  await page.goto(url, { waitUntil: "domcontentloaded" })
  await expect(page.locator("header")).toContainText("Cmdr Smoke Alpha")
  await expect(page.getByRole("region", { name: "Connection state" })).toContainText("Web")
  await expect(page.getByRole("region", { name: "Combat metrics" })).toContainText("Kills")
  await expect(page.getByRole("region", { name: "Health and fuel" })).toContainText("Fuel:")
  await expect(page.getByRole("region", { name: "Warning rail" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText("Kill")
  await assertSanitizedEventFeed(page)
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText("Massacre")
  await expect(page.getByRole("region", { name: "Journal source" })).toContainText(
    "Configured journal folder",
  )
  await expect(page.getByRole("region", { name: "Matrix status" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Web status" })).toContainText("Running")

  const bodyText = await page.locator("body").innerText()
  assertTextAbsent(bodyText, "fixture-smoke-access-token")
  assertTextAbsent(bodyText, "private-journal-root")
  assertTextAbsent(bodyText, "Journal.2035")

  console.log("VISIBLE: Cmdr Smoke Alpha")
  console.log("VISIBLE: Combat metrics / Health and fuel / Warning rail")
  console.log("VISIBLE: Recent event feed / Mission progress / Journal source")
  console.log("VISIBLE: Matrix status / Web status / Connection state")
  console.log("SANITIZED: no Matrix token, private folder, or raw Journal line visible")
  console.log("SANITIZED_STATUS_GLYPHS: no emoji or status pictographs in Recent event feed")

  if (scenario === "responsive") {
    if (responsiveDir === undefined) {
      throw new Error("responsive requires --responsive-dir")
    }
    await captureResponsive(url, responsiveDir)
  } else if (scenario === "buffered-events") {
    if (screenshot === undefined) {
      throw new Error("buffered-events requires --screenshot")
    }
    if (appendJournal === undefined) {
      throw new Error("buffered-events requires --append-journal")
    }
    await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
      "Kill: Viper",
    )
    console.log("BUFFERED_BEFORE_APPEND: Kill: Viper")
    appendFileSync(
      appendJournal,
      '\n{"timestamp":"2035-01-03T10:08:00Z","event":"FactionKillBond","Reward":12000,"AwardingFaction":"Fixture Navy","VictimFaction":"Practice Raiders"}\n',
      "utf8",
    )
    await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText(
      "Kill: Bond",
      { timeout: 10_000 },
    )
    console.log("LIVE_AFTER_APPEND: Kill: Bond")
  } else if (scenario !== "live-dashboard") {
    throw new Error(`Unsupported smoke scenario: ${scenario}`)
  } else if (screenshot === undefined) {
    throw new Error("live-dashboard requires --screenshot")
  }

  if (screenshot !== undefined) {
    await page.screenshot({ path: screenshot, fullPage: true })
    console.log(`SCREENSHOT: ${screenshot}`)
  }
} finally {
  await browser.close()
}

function parseArgs(args) {
  const parsed = new Map()
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index]
    const value = args[index + 1]
    if (key === undefined || value === undefined || !key.startsWith("--")) {
      throw new Error("Expected --key value arguments")
    }
    parsed.set(key.slice(2), value)
  }
  return parsed
}

function requireOption(optionsMap, key) {
  const value = optionsMap.get(key)
  if (value === undefined || value.length === 0) {
    throw new Error(`Missing --${key}`)
  }
  return value
}

function assertTextAbsent(text, forbidden) {
  if (text.includes(forbidden)) {
    throw new Error(`Visible page text leaked forbidden value: ${forbidden}`)
  }
}

async function captureResponsive(pageUrl, outputDir) {
  for (const width of [375, 768, 1280]) {
    await page.setViewportSize({ width, height: 900 })
    await page.goto(pageUrl, { waitUntil: "domcontentloaded" })
    await expect(page.locator("header")).toContainText("Cmdr Smoke Alpha")
    await expect(page.getByRole("region", { name: "Recent event feed" })).toContainText("Kill")
    await assertSanitizedEventFeed(page)
    const overflow = await page.evaluate(() => ({
      clientWidth: document.documentElement.clientWidth,
      scrollWidth: document.documentElement.scrollWidth,
    }))
    const screenshotPath = `${outputDir}/task-10-responsive-${width}.png`
    await page.screenshot({ path: screenshotPath, fullPage: true })
    const overflowing = overflow.scrollWidth > overflow.clientWidth
    console.log(
      `RESPONSIVE: width=${width} clientWidth=${overflow.clientWidth} scrollWidth=${overflow.scrollWidth} horizontalOverflow=${overflowing}`,
    )
    console.log(`SCREENSHOT: ${screenshotPath}`)
    if (overflowing) {
      throw new Error(`Viewport ${width} has horizontal overflow`)
    }
  }
}

async function assertSanitizedEventFeed(page) {
  const eventFeed = page.getByRole("region", { name: "Recent event feed" })
  await expect(eventFeed).toContainText("runtime status")
  await expect(eventFeed).toContainText(/Kills .+\/h uptime .+ missions \d+\/\d+/u)
  const eventFeedText = await eventFeed.innerText()
  assertNoStatusGlyphs(eventFeedText)
  console.log(`EVENT_FEED_TEXT: ${eventFeedText.replaceAll(/\s+/g, " ").trim()}`)
}

function assertNoStatusGlyphs(text) {
  const forbiddenGlyphPattern = /[\p{Extended_Pictographic}\u2300-\u23ff\u2600-\u27bf]/u
  const match = forbiddenGlyphPattern.exec(text)
  if (match?.[0] !== undefined) {
    throw new Error(`Visible event feed text leaked emoji/status glyph: ${match[0]}`)
  }
}
