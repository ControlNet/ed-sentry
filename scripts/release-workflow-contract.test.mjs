import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { test } from "node:test"

test("CI workflow runs for branches and pull requests, not tag pushes", async () => {
  const workflow = await readFile(".github/workflows/ci.yml", "utf8")

  assert.match(workflow, /push:\n\s+branches:\n\s+- ['"]\*\*['"]/u)
  assert.match(workflow, /pull_request:/u)
})

test("release workflow gates on tag version and uploads only smartrelease-compatible assets", async () => {
  const workflow = await readFile(".github/workflows/release.yml", "utf8")

  assert.match(workflow, /contents: write/u)
  assert.match(workflow, /node scripts\/sync-release-version\.mjs --check-tag "\$\{\{ github\.ref_name \}\}"/u)
  assert.match(workflow, /scripts\/package-windows-gnu\.sh/u)
  assert.match(workflow, /scripts\/package-linux-x64\.sh/u)
  assert.match(workflow, /ed-sentry-v\$\{\{ steps\.version\.outputs\.version \}\}-windows-x64\.zip/u)
  assert.match(workflow, /ed-sentry-v\$\{\{ steps\.version\.outputs\.version \}\}-linux-x64\.zip/u)
  assert.doesNotMatch(workflow, /ed-sentry_x86_64-unknown-linux-gnu\.tar\.gz/u)
  assert.doesNotMatch(workflow, /ed-sentry_x86_64-pc-windows-msvc\.zip/u)
})
