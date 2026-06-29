import { readdir, readFile } from "node:fs/promises"
import path from "node:path"
import { expect, test } from "@playwright/test"

const forbiddenDisclaimerPatterns = [
  /no[-\s]?sla/iu,
  /best[-\s]?effort/iu,
  /dev[-\s]?only/iu,
  /development[-\s]?only/iu,
] as const

test("@tunnel-scope keeps Systems config behind tunnel auth and leaves telemetry visible", async ({
  page,
}) => {
  await page.goto("/?mock_state=tunnel_auth_required")

  await expect(page.getByRole("heading", { name: "Telemetry Interface" })).toBeVisible()
  await expect(page.locator("[data-service-node='Tunnel']")).toBeVisible()

  await page.getByRole("button", { name: /Systems/u }).click()

  await expect(page.getByRole("region", { name: "Tunnel config authentication" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Config editor" })).toHaveCount(0)
  await expect(page.getByRole("spinbutton", { name: "Port", exact: true })).toHaveCount(0)

  await page.getByLabel("Tunnel config password").fill("fixture tunnel password")
  await page.getByRole("button", { name: "Unlock Systems" }).click()

  await expect(page.getByRole("region", { name: "Config editor" })).toBeVisible()
  await expect(page.getByRole("region", { name: "Tunnel settings" })).toBeVisible()
})

test("@tunnel-scope does not render Cloudflare disclaimer copy in tunnel UI states", async ({
  page,
}) => {
  for (const mockState of ["tunnel_running", "tunnel_start", "tunnel_retryable_error"] as const) {
    await page.goto(`/?mock_state=${mockState}`)
    const bodyText = (await page.locator("body").textContent()) ?? ""
    for (const pattern of forbiddenDisclaimerPatterns) {
      expect(bodyText).not.toMatch(pattern)
    }
  }
})

test("@tunnel-scope keeps React source free of Cloudflare disclaimer copy", async () => {
  const uiSourceFiles = await sourceFiles(path.join(process.cwd(), "src"))

  for (const filePath of uiSourceFiles) {
    const source = await readFile(filePath, "utf8")
    for (const pattern of forbiddenDisclaimerPatterns) {
      expect(source, filePath).not.toMatch(pattern)
    }
  }
})

async function sourceFiles(directory: string): Promise<readonly string[]> {
  const entries = await readdir(directory, { withFileTypes: true })
  const files: string[] = []
  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await sourceFiles(entryPath)))
      continue
    }
    if (entry.isFile() && /\.(?:ts|tsx|css)$/u.test(entry.name)) {
      files.push(entryPath)
    }
  }
  return files
}
