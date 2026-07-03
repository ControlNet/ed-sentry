import { expect, test } from "@playwright/test"

test("@mission-header shows shared kill progress and ETA", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByTestId("telemetry-active-mission-count")).toHaveText("ACTIVE 2")
  const killProgress = page.getByTestId("telemetry-mission-kill-progress")

  await expect(killProgress).toContainText("24/36")
  await expect(killProgress.getByRole("progressbar", { name: "Progress" })).toHaveAttribute(
    "aria-valuenow",
    "24",
  )
  await expect(page.getByTestId("telemetry-mission-kill-eta")).toContainText("28m")
  await expect(page.getByTestId("telemetry-mission-kill-eta")).not.toContainText("left")
})
