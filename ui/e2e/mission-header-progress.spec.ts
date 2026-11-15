import { expect, test } from "@playwright/test"

test("@mission-header shows shared kill progress and ETA", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByTestId("telemetry-active-mission-count")).toHaveText("2")
  await expect(page.getByTestId("telemetry-mission-kill-progress")).toHaveText("24/36")
  await expect(page.getByTestId("telemetry-mission-kill-eta")).toHaveText("ETA 28m")
  await expect(page.locator(".seg36.s-on")).toHaveCount(24)
  await expect(page.getByRole("region", { name: "Mission progress" })).toContainText(
    "12 kills left",
  )
})

test("@since-last-kill counts up against the no-kills threshold", async ({ page }) => {
  await page.goto("/")

  const panel = page.getByRole("region", { name: "Since last kill" })
  await expect(panel).toContainText("No-kills warning 20m")
  const first = await page.getByTestId("since-last-kill").textContent()
  expect(first).toMatch(/^\d+m \d{2}s$/u)
  await expect(page.getByTestId("since-last-kill")).not.toHaveText(first ?? "", { timeout: 3_000 })
})
