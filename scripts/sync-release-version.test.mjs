import assert from "node:assert/strict"
import { mkdir, readFile, writeFile } from "node:fs/promises"
import { join } from "node:path"
import { test } from "node:test"
import { syncReleaseVersion } from "./sync-release-version.mjs"

test("syncReleaseVersion updates Tauri metadata from the root Cargo version", async (t) => {
  const repoRoot = await t.testContext?.tmpDir?.() ?? await createTempDir(t)
  await writeFixtureRepo(repoRoot, "0.2.3")

  const result = await syncReleaseVersion({ repoRoot, tagName: "v0.2.3" })

  assert.equal(result.version, "0.2.3")
  assert.equal(
    await readFile(join(repoRoot, "ui", "src-tauri", "Cargo.toml"), "utf8"),
    '[package]\nname = "ed-sentry"\nversion = "0.2.3"\n',
  )
  assert.equal(
    await readFile(join(repoRoot, "ui", "src-tauri", "tauri.conf.json"), "utf8"),
    '{\n  "version": "0.2.3"\n}\n',
  )
})

test("syncReleaseVersion rejects a tag that does not match the root Cargo version", async (t) => {
  const repoRoot = await t.testContext?.tmpDir?.() ?? await createTempDir(t)
  await writeFixtureRepo(repoRoot, "0.2.3")

  await assert.rejects(
    syncReleaseVersion({ repoRoot, tagName: "v0.2.4" }),
    /Release tag v0\.2\.4 does not match Cargo\.toml version 0\.2\.3/u,
  )
})

async function createTempDir(t) {
  const { mkdtemp, rm } = await import("node:fs/promises")
  const { tmpdir } = await import("node:os")
  const path = await mkdtemp(join(tmpdir(), "ed-sentry-version-test-"))
  t.after(() => rm(path, { recursive: true, force: true }))
  return path
}

async function writeFixtureRepo(repoRoot, version) {
  await mkdir(join(repoRoot, "ui", "src-tauri"), { recursive: true })
  await writeFile(join(repoRoot, "Cargo.toml"), `[package]\nname = "ed-sentry-core"\nversion = "${version}"\n`)
  await writeFile(
    join(repoRoot, "ui", "src-tauri", "Cargo.toml"),
    '[package]\nname = "ed-sentry"\nversion = "0.0.0"\n',
  )
  await writeFile(join(repoRoot, "ui", "src-tauri", "tauri.conf.json"), '{\n  "version": "0.0.0"\n}\n')
}
