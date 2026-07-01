#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises"
import { dirname, join } from "node:path"
import { fileURLToPath, pathToFileURL } from "node:url"

const scriptDir = dirname(fileURLToPath(import.meta.url))
const defaultRepoRoot = join(scriptDir, "..")

export async function syncReleaseVersion({ repoRoot = defaultRepoRoot, tagName } = {}) {
  const version = await readRootCargoVersion(repoRoot)
  if (tagName !== undefined && tagName !== `v${version}`) {
    throw new Error(`Release tag ${tagName} does not match Cargo.toml version ${version}`)
  }

  await syncTauriCargoVersion(repoRoot, version)
  await syncTauriConfigVersion(repoRoot, version)

  return { version }
}

export async function readRootCargoVersion(repoRoot = defaultRepoRoot) {
  const manifest = await readFile(join(repoRoot, "Cargo.toml"), "utf8")
  const version = /^version = "([0-9]+\.[0-9]+\.[0-9]+(?:[-+][^"]+)?)"$/mu.exec(manifest)?.[1]
  if (version === undefined) {
    throw new Error("Cargo.toml package version is required")
  }
  return version
}

async function syncTauriCargoVersion(repoRoot, version) {
  const path = join(repoRoot, "ui", "src-tauri", "Cargo.toml")
  const manifest = await readFile(path, "utf8")
  const nextManifest = replacePackageVersion(manifest, version, "ui/src-tauri/Cargo.toml")
  await writeFile(path, nextManifest)
}

async function syncTauriConfigVersion(repoRoot, version) {
  const path = join(repoRoot, "ui", "src-tauri", "tauri.conf.json")
  const config = JSON.parse(await readFile(path, "utf8"))
  config.version = version
  await writeFile(path, `${JSON.stringify(config, null, 2)}\n`)
}

function replacePackageVersion(manifest, version, label) {
  const packageVersionPattern = /(^\[package\][\s\S]*?^version = ")([^"]+)(")/mu
  if (!packageVersionPattern.test(manifest)) {
    throw new Error(`${label} package version is required`)
  }
  return manifest.replace(packageVersionPattern, `$1${version}$3`)
}

function parseArgs(args) {
  const options = { repoRoot: defaultRepoRoot, tagName: undefined, printVersion: false }
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index]
    switch (arg) {
      case "--repo-root": {
        index += 1
        const value = args[index]
        if (value === undefined) {
          throw new Error("--repo-root requires a value")
        }
        options.repoRoot = value
        break
      }
      case "--check-tag": {
        index += 1
        const value = args[index]
        if (value === undefined) {
          throw new Error("--check-tag requires a value")
        }
        options.tagName = value
        break
      }
      case "--print-version":
        options.printVersion = true
        break
      default:
        throw new Error(`Unknown argument: ${arg}`)
    }
  }
  return options
}

async function main() {
  const options = parseArgs(process.argv.slice(2))
  if (options.printVersion) {
    process.stdout.write(`${await readRootCargoVersion(options.repoRoot)}\n`)
    return
  }
  const result = await syncReleaseVersion(options)
  process.stdout.write(`Synced release version ${result.version}\n`)
}

if (process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    const message = error instanceof Error ? error.message : String(error)
    console.error(message)
    process.exit(1)
  })
}
