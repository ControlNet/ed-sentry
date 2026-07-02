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
command otherwise.

Regression test: `scripts/generate-brand-assets.test.mjs` covers Windows
`PNPM_HOME` resolution and the missing-shim fallback.

Useful verification commands:

```bash
node --test scripts/*.test.mjs
pnpm --dir ui exec biome check ../scripts/generate-brand-assets.mjs ../scripts/generate-brand-assets.test.mjs
pnpm --dir ui build
./scripts/package-windows-gnu.sh
```
