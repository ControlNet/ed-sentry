import assert from "node:assert/strict"
import { execFile } from "node:child_process"
import { promisify } from "node:util"
import { test } from "node:test"

const execFileAsync = promisify(execFile)

test("Windows package contract includes versioned release name and required files", async () => {
  const { stdout } = await execFileAsync("bash", ["scripts/package-windows-gnu.sh", "--test-package-contract"], {
    cwd: new URL("..", import.meta.url),
  })

  assert.match(stdout, /ed-sentry-v0\.1\.0-windows-x64\.zip/u)
  assert.match(stdout, /README\.md/u)
  assert.match(stdout, /LICENSE/u)
  assert.match(stdout, /tools\/cloudflared\/cloudflared\.exe/u)
})

test("Linux package contract includes versioned release name and required files", async () => {
  const { stdout } = await execFileAsync("bash", ["scripts/package-linux-x64.sh", "--test-package-contract"], {
    cwd: new URL("..", import.meta.url),
  })

  assert.match(stdout, /ed-sentry-v0\.1\.0-linux-x64\.zip/u)
  assert.match(stdout, /ed-sentry-core/u)
  assert.match(stdout, /README\.md/u)
  assert.match(stdout, /LICENSE/u)
  assert.match(stdout, /webui\/index\.html/u)
})
