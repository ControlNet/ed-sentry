import { readFile } from "node:fs/promises"
import { expect, test } from "@playwright/test"
import { z } from "zod"

const tauriCapabilitySchema = z.object({
  permissions: z.array(z.string()),
})

const tauriConfigSchema = z.object({
  app: z.object({
    windows: z.array(
      z.object({
        decorations: z.boolean().optional(),
      }),
    ),
  }),
})

test("@tauri-window-chrome enables frameless drag and window controls", async () => {
  const capability = tauriCapabilitySchema.parse(
    JSON.parse(
      await readFile(new URL("../src-tauri/capabilities/default.json", import.meta.url), "utf8"),
    ),
  )
  const tauriConfig = tauriConfigSchema.parse(
    JSON.parse(await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8")),
  )
  const shellSource = await readFile(
    new URL("../src/components/dashboard/dashboard-shell.tsx", import.meta.url),
    "utf8",
  )

  expect(tauriConfig.app.windows[0]?.decorations).toBe(false)
  expect(capability.permissions).toEqual(
    expect.arrayContaining([
      "core:window:allow-close",
      "core:window:allow-minimize",
      "core:window:allow-start-dragging",
      "core:window:allow-toggle-maximize",
    ]),
  )
  expect(shellSource).toContain("startDragging()")
  expect(shellSource).toContain("currentWindow.minimize()")
  expect(shellSource).toContain("currentWindow.toggleMaximize()")
  expect(shellSource).toContain("currentWindow.close()")
  expect(shellSource).toContain("shouldStartWindowDrag")
  expect(shellSource).toContain("onPointerDownCapture")
  expect(shellSource).toContain('data-titlebar-drag-region="primary-nav"')
  expect(shellSource).toContain('data-titlebar-drag-region="brand-mark"')
  expect(shellSource).toContain('data-titlebar-drag-region="brand-label"')
  expect(shellSource).toContain('data-titlebar-drag-region="status-label"')
  expect(shellSource).toContain('data-titlebar-no-drag="workspace-tab"')
  expect(shellSource).toContain('data-titlebar-no-drag="window-control"')
  expect(shellSource).toContain("readTitlebarDragDebugFlag")
})

test("@tauri-window-chrome visualizes titlebar drag hitmap", async ({ page }) => {
  await page.goto("/?debug_titlebar_drag=1")

  await expect(page.locator("[data-titlebar-drag-debug='true']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='primary-nav']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='brand']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='brand-mark']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='brand-label']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='status']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='status-label']")).toBeVisible()
  await expect(page.locator("[data-titlebar-no-drag='workspace-tab']")).toHaveCount(4)
  await expect(page.locator("[data-titlebar-no-drag='window-control']")).toHaveCount(3)

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/titlebar-drag-hitmap.png",
    fullPage: true,
  })
})
