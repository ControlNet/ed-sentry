import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { test } from "node:test"

const windowsUrl =
  "https://smartrelease.bytedream.dev/github/ControlNet/ed-sentry/ed-sentry-v{major}.{minor}.{patch}-windows-x64.zip"
const linuxUrl =
  "https://smartrelease.bytedream.dev/github/ControlNet/ed-sentry/ed-sentry-v{major}.{minor}.{patch}-linux-x64.zip"

test("README exposes bytedream smartrelease download URLs and visuals", async () => {
  const readme = await readFile("README.md", "utf8")

  assert.match(readme, new RegExp(escapeRegExp(windowsUrl), "u"))
  assert.match(readme, new RegExp(escapeRegExp(linuxUrl), "u"))
  assert.match(readme, /docs\/images\/dashboard\.png/u)
  assert.match(readme, /docs\/images\/logo\.png/u)
})

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")
}
