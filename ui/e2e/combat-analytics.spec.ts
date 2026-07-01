import { expect, test } from "@playwright/test"

test("@combat-analytics displays CLI-aligned total session rates", async ({ page }) => {
  await page.goto("/")

  const combatAnalytics = page.getByRole("region", { name: "Combat Analytics" })

  await expect(combatAnalytics).toContainText("KILL RATE / HR")
  await expect(combatAnalytics).toContainText("25.7/h")
  await expect(combatAnalytics).not.toContainText("31.2/h")
  await expect(combatAnalytics).toContainText("SCAN RATE / HR")
  await expect(combatAnalytics).toContainText("8.6/h")
  await expect(combatAnalytics).not.toContainText("7.9/h")
})

test("@combat-analytics displays CLI idle rate marker when no events exist", async ({ page }) => {
  await page.goto("/?mock_state=empty")

  const combatAnalytics = page.getByRole("region", { name: "Combat Analytics" })

  await expect(combatAnalytics).toContainText("KILL RATE / HR-/h")
  await expect(combatAnalytics).toContainText("SCAN RATE / HR-/h")
})

test("@combat-analytics refreshes total session rates while the dashboard is open", async ({
  page,
}) => {
  await page.goto("/?mock_state=live_rates")

  const combatAnalytics = page.getByRole("region", { name: "Combat Analytics" })

  await expect(combatAnalytics).toContainText("25.7/h")
  await expect(combatAnalytics).toContainText("8.6/h")
  await expect(combatAnalytics).toContainText("25.1/h", { timeout: 3_000 })
  await expect(combatAnalytics).toContainText("8.4/h")
})
