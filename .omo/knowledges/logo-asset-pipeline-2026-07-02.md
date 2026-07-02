# Logo asset pipeline

- Canonical hand-authored source: `docs/images/logo.svg`.
- Generated assets are produced by `node scripts/generate-brand-assets.mjs`.
- The script runs Tauri's icon generator and outputs desktop/mobile icons into `ui/src-tauri/icons/`.
- The same run syncs `ui/public/favicon.ico`, `ui/public/logo.png`, and `docs/images/logo.png` from the generated Tauri assets.
- `pnpm --dir ui build` runs `pnpm assets:brand` first, so Windows/Linux package scripts refresh icons through the normal WebUI build path.
- GUI chrome uses `/logo.png`; README continues to reference `docs/images/logo.png`.
