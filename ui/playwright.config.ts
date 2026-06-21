import { defineConfig, devices } from "@playwright/test"

export default defineConfig({
  testDir: "./e2e",
  outputDir: "../.omo/evidence/gui-webui-tauri/playwright-results",
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://127.0.0.1:4173",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "VITE_DASHBOARD_ADAPTER=mock pnpm build && pnpm preview --host 127.0.0.1 --port 4173",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: false,
    timeout: 120_000,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
})
