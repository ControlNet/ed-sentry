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
        label: z.string().optional(),
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
  const loadingSource = await readFile(
    new URL("../src/components/dashboard/loading-screen.tsx", import.meta.url),
    "utf8",
  )

  expect(tauriConfig.app.windows[0]?.decorations).toBe(false)
  expect(tauriConfig.app.windows[0]?.label).toBe("main")
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
  expect(shellSource).toContain("data-tauri-drag-region={tauriDragRegion}")
  expect(shellSource).toContain('data-titlebar-drag-region="primary-nav"')
  expect(shellSource).toContain('data-titlebar-drag-region="brand-mark"')
  expect(shellSource).toContain('data-titlebar-drag-region="brand-label"')
  expect(shellSource).toContain('data-titlebar-drag-region="status-label"')
  expect(shellSource).toContain('data-titlebar-no-drag="workspace-tab"')
  expect(shellSource).toContain('data-titlebar-no-drag="window-control"')
  expect(shellSource).toContain("readTitlebarDragDebugFlag")
  expect(loadingSource).toContain("startDragging()")
  expect(loadingSource).toContain('data-titlebar-drag-region="loading-titlebar"')
  expect(loadingSource).toContain("data-tauri-drag-region={tauriDragRegion}")
  expect(loadingSource).toContain("readTitlebarDragDebugFlag")
})

test("@tauri-window-chrome visualizes titlebar drag hitmap", async ({ page }) => {
  await page.goto("/?debug_titlebar_drag=1")

  await expect(page.locator("[data-titlebar-drag-debug='true']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='primary-nav']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='primary-nav']")).toHaveAttribute(
    "data-tauri-drag-region",
    "",
  )
  await expect(page.locator("[data-titlebar-drag-region='primary-nav-list']")).toHaveAttribute(
    "data-tauri-drag-region",
    "",
  )
  await expect(page.locator("[data-titlebar-drag-region='brand']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='brand-mark']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='brand-label']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='status']")).toBeVisible()
  await expect(page.locator("[data-titlebar-drag-region='status-label']")).toBeVisible()
  await expect(page.locator("[data-titlebar-no-drag='workspace-tab']")).toHaveCount(4)
  await expect(page.locator("[data-titlebar-no-drag='window-controls']")).toHaveCount(1)
  await expect(page.locator("[data-titlebar-no-drag='window-control']")).toHaveCount(3)

  await page.screenshot({
    path: "../.omo/evidence/gui-webui-tauri/titlebar-drag-hitmap.png",
    fullPage: true,
  })
})

test("@tauri-window-chrome does not block window setup on desktop runtime startup", async () => {
  const desktopGuiSource = await readFile(
    new URL("../../src/desktop_gui/mod.rs", import.meta.url),
    "utf8",
  )
  const launcherSource = await readFile(
    new URL("../src-tauri/src/main.rs", import.meta.url),
    "utf8",
  )
  const rootBuildSource = await readFile(new URL("../../build.rs", import.meta.url), "utf8")

  expect(desktopGuiSource).not.toContain("block_on(DesktopRuntime::start")
  expect(desktopGuiSource).toContain("spawn_desktop_runtime")
  expect(launcherSource).toContain('command.arg("--gui")')
  expect(rootBuildSource).toContain('join("ui").join("src-tauri")')
  expect(rootBuildSource).toContain("tauri_build::build()")
})
