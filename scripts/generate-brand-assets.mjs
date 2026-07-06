#!/usr/bin/env node
import { spawnSync } from "node:child_process"
import { existsSync } from "node:fs"
import { copyFile, stat } from "node:fs/promises"
import path, { dirname, join, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const scriptDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = join(scriptDir, "..")
const sourceLogo = join(repoRoot, "docs", "images", "logo.svg")
const tauriIconDir = join(repoRoot, "ui", "src-tauri", "icons")
const tauriIconPng = resolveGeneratedLogoPngPath()
const tauriIconIco = join(tauriIconDir, "icon.ico")
const docsLogoPng = join(repoRoot, "docs", "images", "logo.png")
const webLogoPng = join(repoRoot, "ui", "public", "logo.png")
const webFavicon = join(repoRoot, "ui", "public", "favicon.ico")

async function main() {
  await requireFile(sourceLogo, "source logo SVG")
  run(resolvePnpmCommand(), resolveTauriIconArgs())
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
    ...resolveSpawnOptions(),
  })
  if (result.error !== undefined) {
    throw result.error
  }
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with status ${result.status}`)
  }
}

export function resolveSpawnOptions(platform = process.platform) {
  return {
    cwd: repoRoot,
    stdio: "inherit",
    shell: platform === "win32",
  }
}

export function resolveTauriIconArgs(sourcePath = sourceLogo, outputDir = tauriIconDir) {
  return ["--dir", "ui", "tauri", "icon", sourcePath, "--output", outputDir]
}

export function resolveGeneratedLogoPngPath(outputDir = tauriIconDir) {
  return join(outputDir, "128x128@2x.png")
}

export function resolvePnpmCommand(
  env = process.env,
  platform = process.platform,
  fileExists = existsSync,
) {
  const executable = platform === "win32" ? "pnpm.cmd" : "pnpm"
  const packageManagerHome = env.PNPM_HOME
  if (packageManagerHome !== undefined && packageManagerHome.trim() !== "") {
    const pathModule = platform === "win32" ? path.win32 : path.posix
    const candidate = pathModule.join(packageManagerHome, executable)
    if (fileExists(candidate)) {
      return candidate
    }
  }
  return executable
}

if (process.argv[1] !== undefined && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main().catch((error) => {
    const message = error instanceof Error ? error.message : String(error)
    console.error(message)
    process.exit(1)
  })
}
