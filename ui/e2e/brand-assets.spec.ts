import { expect, test } from "@playwright/test"

test("@brand-assets serves the generated logo and favicon", async ({ page }) => {
  await page.goto("/")

  await expect(page.locator("link[rel='icon']")).toHaveAttribute("href", "/favicon.ico")
  await expect(page.locator("link[rel='preload'][as='image']")).toHaveAttribute("href", "/logo.png")

  const faviconResponse = await page.request.get("/favicon.ico")
  expect(faviconResponse.ok()).toBe(true)
  expect(faviconResponse.headers()["content-type"]).toContain("image/")
  expect((await faviconResponse.body()).length).toBeGreaterThan(1_000)

  const logoResponse = await page.request.get("/logo.png")
  expect(logoResponse.ok()).toBe(true)
  expect(logoResponse.headers()["content-type"]).toContain("image/")
  expect((await logoResponse.body()).length).toBeGreaterThan(1_000)
})
