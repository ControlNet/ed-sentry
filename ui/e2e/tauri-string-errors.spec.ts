import { expect, test } from "@playwright/test"
import { createTauriDashboardAdapter, type EditableConfigUpdate } from "@/adapters/dashboard"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { mockConfigUpdate, mockConfigView } from "./adapter-boundary-fixtures"

test("tauri adapter preserves safe string errors from snapshot command rejection", async () => {
  const adapter = createTauriDashboardAdapter({
    loadSnapshot: async () => Promise.reject("Snapshot command failed: runtime unavailable"),
    loadConfig: async () => mockConfigView(),
    saveConfig: async (_update: EditableConfigUpdate) => mockConfigView(),
  })

  await expect(adapter.loadSnapshot()).rejects.toThrow(
    "Snapshot command failed: runtime unavailable",
  )
  await expect(adapter.loadSnapshot()).rejects.not.toThrow("non-Error value")
})

test("tauri adapter redacts sensitive string errors from config load command rejection", async () => {
  const sensitiveConfigKey = ["access", "token"].join("_")
  const adapter = createTauriDashboardAdapter({
    loadSnapshot: async () => mockDashboardSnapshot,
    loadConfig: async () =>
      Promise.reject(
        `Config load failed at /home/commander/.config/ed-sentry/config.toml: ${sensitiveConfigKey} = "fixture-sensitive-value"`,
      ),
    saveConfig: async (_update: EditableConfigUpdate) => mockConfigView(),
  })

  await expect(adapter.loadConfig()).rejects.toThrow("Config load failed at [redacted path]")
  await expect(adapter.loadConfig()).rejects.not.toThrow("/home/commander")
  await expect(adapter.loadConfig()).rejects.not.toThrow("fixture-sensitive-value")
})

test("tauri adapter preserves safe string errors from config save command rejection", async () => {
  const adapter = createTauriDashboardAdapter({
    loadSnapshot: async () => mockDashboardSnapshot,
    loadConfig: async () => mockConfigView(),
    saveConfig: async (_update: EditableConfigUpdate) =>
      Promise.reject("Config save failed: write permission denied"),
  })

  await expect(adapter.saveConfig(mockConfigUpdate())).rejects.toThrow(
    "Config save failed: write permission denied",
  )
  await expect(adapter.saveConfig(mockConfigUpdate())).rejects.not.toThrow("non-Error value")
})
