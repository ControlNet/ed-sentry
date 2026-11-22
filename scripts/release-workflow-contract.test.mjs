import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { test } from "node:test"

test("CI workflow runs for branches and pull requests, not tag pushes", async () => {
  const workflow = await readFile(".github/workflows/ci.yml", "utf8")

  assert.match(workflow, /push:\n\s+branches:\n\s+- ['"]\*\*['"]/u)
  assert.match(workflow, /pull_request:/u)
  assert.match(workflow, /cargo clean/u)
  assert.doesNotMatch(workflow, /\n\s+target\/\n/u)
})

test("CI and release workflows run the frontend lint, unit tests, and release contract tests", async () => {
  for (const path of [".github/workflows/ci.yml", ".github/workflows/release.yml"]) {
    const workflow = await readFile(path, "utf8")
    assert.match(workflow, /run: node --test scripts\/\*\.test\.mjs\n/u, path)
    assert.match(workflow, /run: pnpm --dir ui lint\n/u, path)
    assert.match(workflow, /run: pnpm --dir ui test\n/u, path)
  }
})

test("release workflow gates on tag version and uploads only smartrelease-compatible assets", async () => {
  const workflow = await readFile(".github/workflows/release.yml", "utf8")

  assert.match(workflow, /contents: write/u)
  assert.match(workflow, /node scripts\/sync-release-version\.mjs --check-tag "\$\{\{ github\.ref_name \}\}"/u)
  assert.match(workflow, /scripts\/package-windows-gnu\.sh/u)
  assert.match(workflow, /scripts\/package-linux-x64\.sh/u)
  assert.match(workflow, /libwebkit2gtk-4\.1-dev/u)
  assert.match(workflow, /pnpm --dir ui build/u)
  assert.match(workflow, /cargo clean/u)
  assert.doesNotMatch(workflow, /\n\s+(?:ui\/src-tauri\/)?target\/\n/u)
  assert.match(workflow, /ed-sentry-v\$\{\{ steps\.version\.outputs\.version \}\}-windows-x64\.zip/u)
  assert.match(workflow, /ed-sentry-v\$\{\{ steps\.version\.outputs\.version \}\}-linux-x64\.zip/u)
  assert.doesNotMatch(workflow, /ed-sentry-x86_64-unknown-linux-gnu\.tar\.gz/u)
  assert.doesNotMatch(workflow, /ed-sentry-x86_64-pc-windows-msvc\.zip/u)
})
