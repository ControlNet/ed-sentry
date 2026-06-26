# WebUI favicon asset contract

- The browser WebUI favicon is served from `/favicon.ico` via Vite `public` assets.
- Keep `ui/public/favicon.ico` present so `pnpm --dir ui build` copies it to `ui/dist/favicon.ico` and release packaging copies it into `dist/ed-sentry/webui/favicon.ico`.
- The favicon intentionally reuses `ui/src-tauri/icons/icon.ico` so browser tabs and the desktop launcher share the same app icon.
