import assert from "node:assert/strict"
import { test } from "node:test"

import { resolvePnpmCommand, resolveSpawnOptions } from "./generate-brand-assets.mjs"

test("brand asset generator resolves pnpm from PNPM_HOME on Windows", () => {
  const checked = []
  const command = resolvePnpmCommand(
    {
      PNPM_HOME: "C:\\Users\\runneradmin\\setup-pnpm\\node_modules\\.bin",
    },
    "win32",
    (path) => {
      checked.push(path)
      return true
    },
  )

  assert.equal(command, "C:\\Users\\runneradmin\\setup-pnpm\\node_modules\\.bin\\pnpm.cmd")
  assert.deepEqual(checked, ["C:\\Users\\runneradmin\\setup-pnpm\\node_modules\\.bin\\pnpm.cmd"])
})

test("brand asset generator falls back to bare pnpm command when PNPM_HOME has no shim", () => {
  const command = resolvePnpmCommand(
    {
      PNPM_HOME: "/home/ubuntu/.local/share/pnpm",
    },
    "linux",
    () => false,
  )

  assert.equal(command, "pnpm")
})

test("brand asset generator uses a shell for Windows command shims", () => {
  assert.equal(resolveSpawnOptions("win32").shell, true)
  assert.equal(resolveSpawnOptions("linux").shell, false)
})
