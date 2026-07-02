# Windows CI pnpm Spawn

The CI run for `05ec39a` failed on `Rust (windows-latest)` during the
`Build WebUI assets` step. Ubuntu passed.

Failure log:

```text
pnpm --dir ui build
> pnpm assets:brand && tsc -b && vite build
> node ../scripts/generate-brand-assets.mjs
spawnSync pnpm ENOENT
```

Mechanism: the outer PowerShell step can resolve `pnpm`, but the Node script's
`spawnSync("pnpm", ...)` does not reliably resolve the GitHub Actions pnpm shim
on Windows. The CI environment provides `PNPM_HOME`, so the script should prefer
`PNPM_HOME/pnpm.cmd` on Windows when that shim exists, falling back to the bare
command otherwise. Directly spawning the `.cmd` shim without a shell then failed
with `spawnSync ... pnpm.cmd EINVAL`, so Windows command-shim execution must use
`shell: true`.

Regression test: `scripts/generate-brand-assets.test.mjs` covers Windows
`PNPM_HOME` resolution, the missing-shim fallback, and Windows shell execution
for command shims.

Useful verification commands:

```bash
node --test scripts/*.test.mjs
pnpm --dir ui exec biome check ../scripts/generate-brand-assets.mjs ../scripts/generate-brand-assets.test.mjs
pnpm --dir ui build
./scripts/package-windows-gnu.sh
```
