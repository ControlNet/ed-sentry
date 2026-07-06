# Logo asset pipeline

- Canonical hand-authored source: `docs/images/logo.svg`.
- Generated assets are produced by `node scripts/generate-brand-assets.mjs`.
- The script runs Tauri's icon generator and outputs desktop/mobile icons into `ui/src-tauri/icons/`.
- The same run syncs `ui/public/favicon.ico`, `ui/public/logo.png`, and `docs/images/logo.png` from the generated Tauri assets.
- `pnpm --dir ui build` runs `pnpm assets:brand` first, so Windows/Linux package scripts refresh icons through the normal WebUI build path.
- GUI chrome uses generated `/logo.png`; README references `docs/images/logo.svg` directly.
- Generated icon outputs are intentionally ignored and untracked: `docs/images/logo.png`, `ui/public/logo.png`, `ui/public/favicon.ico`, and `ui/src-tauri/icons/`.
- Fresh CI checkouts may not have ignored output directories such as `ui/public/`; the generator must create parent directories before writing generated files.
