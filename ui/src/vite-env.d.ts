interface ImportMetaEnv {
  readonly VITE_DASHBOARD_ADAPTER?: "mock" | "web"
  readonly VITE_ED_SENTRY_API_BASE_URL?: string
  readonly VITE_ED_SENTRY_BUILD_VERSION: string
  readonly VITE_DISABLE_REACT_DEVTOOLS?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
