import { spawn } from "node:child_process"

const rawArgs = process.argv.slice(2)
const args = rawArgs[0] === "--" ? rawArgs.slice(1) : rawArgs
const command = process.platform === "win32" ? "playwright.cmd" : "playwright"
const child = spawn(command, ["test", ...args], {
  stdio: "inherit",
})

child.on("exit", (code, signal) => {
  if (signal !== null) {
    process.kill(process.pid, signal)
    return
  }
  process.exitCode = code ?? 1
})

child.on("error", (error) => {
  console.error(error.message)
  process.exitCode = 1
})
