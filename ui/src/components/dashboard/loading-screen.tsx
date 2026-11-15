import { AppFrame } from "./g1-ui"

type LoadingScreenProps = {
  readonly detail: string
  readonly isTauri: boolean
}

/**
 * Shown until the first snapshot arrives. The adapter reports no measurable
 * progress, so the bar is indeterminate and the text is the adapter's own
 * connection detail.
 */
export function LoadingScreen({ detail, isTauri }: LoadingScreenProps): React.JSX.Element {
  const titlebarDragDebugEnabled = readTitlebarDragDebugFlag()
  const showTauriDragRegion = isTauri || titlebarDragDebugEnabled
  const tauriDragRegion = showTauriDragRegion ? "" : undefined

  return (
    <AppFrame dragDebug={titlebarDragDebugEnabled}>
      <div
        aria-hidden="true"
        style={{ position: "absolute", left: 0, right: 0, top: 0, height: 56, zIndex: 2 }}
        data-tauri-drag-region={tauriDragRegion}
        data-titlebar-drag-region="loading-titlebar"
        onPointerDownCapture={(event) => {
          if (isTauri && event.button === 0) {
            void startWindowDrag()
          }
        }}
      />
      <main
        className="layer"
        style={{
          flex: "1 1 auto",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 56,
          padding: 40,
          flexWrap: "wrap",
        }}
      >
        <div
          style={{ position: "relative", width: 300, height: 300, flexShrink: 0 }}
          aria-hidden="true"
        >
          <div
            className="orb spin"
            style={{ width: 290, height: 290, border: "1px dashed rgba(60,184,255,.35)" }}
          />
          <div
            className="orb"
            style={{ width: 210, height: 210, border: "1px solid rgba(255,122,0,.3)" }}
          />
          <div
            className="orb"
            style={{ width: 130, height: 130, border: "1px solid rgba(60,184,255,.2)" }}
          />
          <div className="orb" style={{ width: 26, height: 26, background: "#ff7a00" }} />
        </div>
        <section
          className="pnl"
          aria-label="Dashboard startup"
          aria-live="polite"
          style={{ width: "min(100%, 560px)" }}
        >
          <h1 className="pt" style={{ marginBottom: 18 }}>
            <span>
              <span className="or">ED Sentry</span>
              <span style={{ margin: "0 10px", color: "#4b525c" }}>|</span>
              <span className="sec2">Starting</span>
            </span>
            <span className="lab">v{import.meta.env.VITE_ED_SENTRY_BUILD_VERSION}</span>
          </h1>
          <h2
            className="loading-status"
            style={{
              margin: "0 0 6px",
              fontSize: 20,
              fontWeight: 500,
              letterSpacing: ".08em",
              textTransform: "uppercase",
              color: "#ff8a1a",
            }}
          >
            Waiting for first snapshot
          </h2>
          <p className="sec2" style={{ margin: "0 0 18px" }} data-testid="loading-detail">
            {detail}
          </p>
          <div className="bar ld-bar" aria-hidden="true">
            <div className="ld-run" />
          </div>
        </section>
      </main>
    </AppFrame>
  )
}

async function startWindowDrag(): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window")
  await getCurrentWindow().startDragging()
}

function readTitlebarDragDebugFlag(): boolean {
  if (typeof window === "undefined") {
    return false
  }
  return new URLSearchParams(window.location.search).get("debug_titlebar_drag") === "1"
}
