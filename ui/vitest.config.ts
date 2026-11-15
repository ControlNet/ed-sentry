import { defineConfig } from "vitest/config"

// Unit tests cover pure view-model helpers only. Worker count is capped because
// the development machine has limited free memory.
export default defineConfig({
  resolve: {
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
    pool: "threads",
    poolOptions: {
      threads: {
        minThreads: 1,
        maxThreads: 2,
      },
    },
  },
})
