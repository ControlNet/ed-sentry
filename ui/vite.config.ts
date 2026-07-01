import { execFileSync } from "node:child_process"
import { readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"
import reactScan from "vite-plugin-react-scan"

const configDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(configDir, "..")

export default defineConfig({
  plugins: [react(), reactScan(), tailwindcss()],
  define: {
    "import.meta.env.VITE_ED_SENTRY_BUILD_VERSION": JSON.stringify(buildVersion()),
  },
  resolve: {
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
})

function buildVersion(): string {
  return `${packageVersion()}-${latestCommitDate()}`
}

function packageVersion(): string {
  const manifest = readFileSync(resolve(repoRoot, "Cargo.toml"), "utf8")
  const version = /^version = "([^"]+)"$/mu.exec(manifest)?.[1]
  if (version === undefined) {
    throw new Error("Cargo.toml package version is required for the WebUI build version")
  }
  return version
}

function latestCommitDate(): string {
  return execFileSync("git", ["log", "-1", "--format=%cd", "--date=format:%Y%m%d"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim()
}
