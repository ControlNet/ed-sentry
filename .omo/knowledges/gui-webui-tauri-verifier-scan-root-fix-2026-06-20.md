# GUI/WebUI/Tauri Verifier Secret Scan Root Fix

Date: 2026-06-21 UTC

The `scripts/verify-gui-webui-tauri.sh` secret scan previously used narrow roots: `ui/src`, `ui/src-tauri`, and `ui/package.json`. That missed feature-relevant untracked non-ignored files under `ui/e2e`, `ui/scripts`, and UI root config files such as Playwright, Vite, Biome, tsconfig, pnpm workspace, and index files.

The verifier now scans the `ui` root and keeps generated/heavy outputs excluded with explicit globs for `ui/node_modules`, `ui/dist`, `ui/playwright-report`, `ui/test-results`, `ui/src-tauri/icons`, and `ui/src-tauri/target`. Root config/docs relevant to packaging/examples are also included when present: `Cargo.lock`, `DESIGN.md`, `config.example.toml`, `rust-toolchain.toml`, and `rustfmt.toml`.

The raw secret match suppression remains intact: `rg` writes matches only to a temporary file, evidence reports sanitized counts/categories, and the verifier exits non-zero if unexpected secret-like lines are greater than zero.

Useful verification artifacts:

- `.omo/evidence/gui-webui-tauri/global-review-code-quality-final-rerun-fix-supplemental-coverage.txt`
- `.omo/evidence/gui-webui-tauri/global-review-code-quality-final-rerun-fix-supplemental-secret-counts.txt`
- `.omo/evidence/gui-webui-tauri/global-review-code-quality-final-rerun-fix-source-review.md`
