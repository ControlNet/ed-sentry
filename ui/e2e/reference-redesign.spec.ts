import { expect, test } from "@playwright/test"

test("@reference-redesign renders the G1 v2 workspace navigation", async ({ page }) => {
  await page.goto("/")

  const nav = page.getByRole("navigation", { name: "Primary" })
  for (const label of ["Dashboard", "Missions", "Events", "Systems", "About"]) {
    await expect(nav.getByRole("button", { name: label, exact: true })).toBeVisible()
  }
  await expect(nav.getByRole("button", { name: "Dashboard", exact: true })).toHaveAttribute(
    "aria-current",
    "page",
  )

  await expect(page.getByRole("heading", { name: "Dashboard", level: 1 })).toBeVisible()
  await expect(page.locator("header")).toContainText("Sanitized System")
  await expect(page.locator("footer")).toContainText("Krait Mk II")

  await nav.getByRole("button", { name: "Missions", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Missions", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Active missions" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Mission details" })).toContainText(
    "Massacre pirates",
  )
  await expect(page.getByRole("region", { name: "Mission timeline" })).toContainText("to expiry")

  await nav.getByRole("button", { name: "Events", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Events", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Event log" })).toContainText("Bounty voucher")

  await nav.getByRole("button", { name: "Systems", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Systems", level: 1 })).toBeVisible()
  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()
  await expect(page.getByLabel("Journal folder")).toHaveAttribute(
    "placeholder",
    "Default journal folder",
  )
  await expect(page.getByLabel("Replace access token")).toHaveValue("")
})

test("@reference-redesign mission selection follows the list and the timeline", async ({
  page,
}) => {
  await page.goto("/")
  await page.getByRole("button", { name: "Missions", exact: true }).click()

  const details = page.getByRole("region", { name: "Mission details" })
  await expect(details).toContainText("Massacre pirates")
  await page
    .getByRole("region", { name: "Active missions" })
    .getByRole("button", { name: /Suppress hostile wing/u })
    .click()
  await expect(details).toContainText("Suppress hostile wing")
  await expect(details).toContainText("4 remaining")

  await page
    .getByRole("region", { name: "Mission timeline" })
    .getByRole("button", { name: /Massacre pirates/u })
    .click()
  await expect(details).toContainText("Massacre pirates")
})

test("@reference-redesign systems index summarises each section", async ({ page }) => {
  await page.goto("/")
  await page.getByRole("button", { name: "Systems", exact: true }).click()

  const index = page.getByRole("complementary", { name: "Configuration index" })
  await expect(index).toContainText("Journal")
  await expect(index).toContainText("Event routing")
  await expect(index).toContainText(/\d+ events · \d+ mention · \d+ off/u)
  await expect(index).toContainText("Autosave enabled")

  await index.getByRole("button", { name: /Monitor/u }).click()
  await expect(page.getByRole("region", { name: "Monitor settings" })).toBeInViewport()
})
