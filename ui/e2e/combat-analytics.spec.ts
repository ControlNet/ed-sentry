import { expect, test } from "@playwright/test"

test("@combat-analytics compares the recent rate with the session average", async ({ page }) => {
  await page.goto("/")

  const kills = page.getByTestId("combat-kills")
  await expect(kills).toContainText("18")
  await expect(kills).toContainText("Last 14:11:19")
  await expect(kills).toContainText("31.2/h")
  await expect(kills).toContainText("25.7/h")
  await expect(kills).toContainText("▲ +21%")

  const scans = page.getByTestId("combat-scans")
  await expect(scans).toContainText("7.9/h")
  await expect(scans).toContainText("8.6/h")
  await expect(scans).toContainText("▼ −8%")

  await expect(page.getByTestId("combat-bounty")).toContainText("4,218,450")
  await expect(page.getByTestId("combat-merits")).toContainText("+3 to report")
})

test("@combat-analytics displays CLI idle rate marker when no events exist", async ({ page }) => {
  await page.goto("/?mock_state=empty")

  const combat = page.getByRole("region", { name: "Combat" })
  await expect(page.getByTestId("combat-kills")).toContainText("-/h")
  await expect(page.getByTestId("combat-scans")).toContainText("-/h")
  await expect(combat).not.toContainText("0.0/h")
  await expect(combat.locator(".chip")).toHaveCount(0)
  await expect(page.getByTestId("since-last-kill")).toHaveText("—")
})

test("@combat-analytics refreshes session rates while the dashboard is open", async ({ page }) => {
  await page.goto("/?mock_state=live_rates")

  const combat = page.getByRole("region", { name: "Combat" })

  await expect(combat).toContainText("25.7/h")
  await expect(combat).toContainText("8.6/h")
  await expect(combat).toContainText("25.1/h", { timeout: 3_000 })
  await expect(combat).toContainText("8.4/h")
})

test("@combat-analytics marks the configured kill-rate warning on the rate scale", async ({
  page,
}) => {
  await page.goto("/")

  const kills = page.getByTestId("combat-kills")
  await expect(kills.locator(".wt")).toHaveCount(2)
  await expect(kills.locator(".axis .rd")).toHaveText("20")
})

test("@live-updates flashes changed values and highlights a newly arrived event", async ({
  page,
}) => {
  await page.goto("/?mock_state=live_feed")

  const eventRows = page.getByTestId("telemetry-event-row")
  const killCount = page.getByTestId("combat-kills").locator(".v")
  await expect(killCount).toHaveText("19")
  await expect(killCount).toHaveClass(/fx-a/u)
  await expect(eventRows.first()).toHaveClass(/fresh/u)
  await expect(page.locator(".ev.fresh")).toHaveCount(1)
  await expect(page.getByTestId("telemetry-mission-kill-progress")).not.toHaveClass(/fx-/u)

  // A remounted view starts without highlights, so switching tabs never replays them.
  await page.getByRole("button", { name: "Events", exact: true }).click()
  await expect(page.getByTestId("event-log-entry").first()).toContainText("14:18:02")
  await expect(page.locator(".ev.fresh")).toHaveCount(0)
})
