#!/usr/bin/env node
import { copyFile, stat } from "node:fs/promises"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { spawnSync } from "node:child_process"

const scriptDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = join(scriptDir, "..")
const sourceLogo = join(repoRoot, "docs", "images", "logo.svg")
const tauriIconDir = join(repoRoot, "ui", "src-tauri", "icons")
const tauriIconPng = join(tauriIconDir, "icon.png")
const tauriIconIco = join(tauriIconDir, "icon.ico")
const docsLogoPng = join(repoRoot, "docs", "images", "logo.png")
const webLogoPng = join(repoRoot, "ui", "public", "logo.png")
const webFavicon = join(repoRoot, "ui", "public", "favicon.ico")

async function main() {
  await requireFile(sourceLogo, "source logo SVG")
  run("pnpm", ["--dir", "ui", "tauri", "icon", "../docs/images/logo.svg", "--output", "src-tauri/icons"])
  await requireFile(tauriIconPng, "generated Tauri PNG icon")
  await requireFile(tauriIconIco, "generated Tauri ICO icon")
  await copyFile(tauriIconPng, docsLogoPng)
  await copyFile(tauriIconPng, webLogoPng)
  await copyFile(tauriIconIco, webFavicon)
  process.stdout.write("Generated brand assets from docs/images/logo.svg\n")
}

async function requireFile(path, label) {
  const file = await stat(path)
  if (!file.isFile()) {
    throw new Error(`Expected ${label}: ${path}`)
  }
}

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
  })
  if (result.error !== undefined) {
    throw result.error
  }
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with status ${result.status}`)
  }
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error)
  console.error(message)
  process.exit(1)
})
