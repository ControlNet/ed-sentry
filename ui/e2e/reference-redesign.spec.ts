import { expect, test } from "@playwright/test"

test("@reference-redesign renders the reference-derived workspace tabs", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByRole("button", { name: /Dashboard/u })).toBeVisible()
  await expect(page.getByRole("button", { name: /Missions/u })).toBeVisible()
  await expect(page.getByRole("button", { name: /Comms Feed/u })).toBeVisible()
  await expect(page.getByRole("button", { name: /Systems/u })).toBeVisible()

  await expect(page.getByRole("heading", { name: /Dashboard Interface/u })).toBeVisible()
  await expect(page.getByRole("region", { name: /Telemetry Link/u })).toContainText(
    "Local Commander",
  )

  await page.getByRole("button", { name: /Missions/u }).click()
  await expect(page.getByRole("heading", { name: /Missions Interface/u })).toBeVisible()
  await expect(page.getByRole("region", { name: /Mission Directory/u })).toBeVisible()
  await expect(page.getByRole("region", { name: /Mission Intel/u })).toContainText(
    "Massacre pirates",
  )

  await page.getByRole("button", { name: /Comms Feed/u }).click()
  await expect(page.getByRole("heading", { name: /Comms Feed Interface/u })).toBeVisible()
  await expect(page.getByRole("region", { name: /System Telemetry Feed/u })).toContainText(
    "Bounty voucher",
  )

  await page.getByRole("button", { name: /Systems/u }).click()
  await expect(page.getByRole("heading", { name: /Systems Interface/u })).toBeVisible()
  await expect(page.getByRole("region", { name: /System Configuration/u })).toBeVisible()
  await expect(page.getByLabel("Journal folder")).toHaveAttribute(
    "placeholder",
    "Default journal folder",
  )
  await expect(page.getByLabel("Replace access token")).toHaveValue("")
})
