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
  assert.match(
    readme,
    /<p align="center">\s*<img src="docs\/images\/logo\.svg" alt="ED Sentry logo" height="120">\s*<\/p>/u,
  )
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/actions\/workflow\/status\/ControlNet\/ed-sentry\/ci\.yml\?branch=master&style=flat-square&label=CI/u)
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/v\/release\/ControlNet\/ed-sentry\?style=flat-square/u)
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/issues\/ControlNet\/ed-sentry\?style=flat-square/u)
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/stars\/ControlNet\/ed-sentry\?style=flat-square/u)
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/forks\/ControlNet\/ed-sentry\?style=flat-square/u)
  assert.match(readme, /https:\/\/img\.shields\.io\/github\/license\/ControlNet\/ed-sentry\?style=flat-square/u)
  assert.match(readme, /https:\/\/github\.com\/PsiPab\/ED-AFK-Monitor/u)
  assert.match(readme, /https:\/\/github\.com\/WarmedxMints\/ODEliteTracker/u)
  await assertFileExists("docs/images/logo.svg")
})

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")
}

async function assertFileExists(path) {
  const file = await readFile(path)
  assert.ok(file.length > 0, `${path} should not be empty`)
}
