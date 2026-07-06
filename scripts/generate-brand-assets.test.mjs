import assert from "node:assert/strict"
import path from "node:path"
import { test } from "node:test"

import {
  resolvePnpmCommand,
  resolveSpawnOptions,
  resolveTauriIconArgs,
} from "./generate-brand-assets.mjs"

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

test("brand asset generator passes absolute source and output paths to Tauri", () => {
  const args = resolveTauriIconArgs()

  assert.equal(args[0], "--dir")
  assert.equal(args[1], "ui")
  assert.equal(args[2], "tauri")
  assert.equal(args[3], "icon")
  assert.ok(path.isAbsolute(args[4]), "source SVG path should be absolute")
  assert.equal(args[5], "--output")
  assert.ok(path.isAbsolute(args[6]), "Tauri icon output path should be absolute")
  assert.match(args[4], /docs[/\\]images[/\\]logo\.svg$/u)
  assert.match(args[6], /ui[/\\]src-tauri[/\\]icons$/u)
})
