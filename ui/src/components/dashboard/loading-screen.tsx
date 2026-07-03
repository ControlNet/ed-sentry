import { EyeOff, Loader2 } from "lucide-react"
import { useEffect, useMemo, useState } from "react"

type LoadingScreenProps = {
  readonly detail: string
  readonly isTauri: boolean
}

const RADIUS = 70
const CIRCUMFERENCE = 2 * Math.PI * RADIUS
const MAX_PENDING_PROGRESS = 96

export function LoadingScreen({ detail, isTauri }: LoadingScreenProps): React.JSX.Element {
  const [progress, setProgress] = useState(0)
  const titlebarDragDebugEnabled = readTitlebarDragDebugFlag()
  const showTauriDragRegion = isTauri || titlebarDragDebugEnabled
  const tauriDragRegion = showTauriDragRegion ? "" : undefined

  useEffect(() => {
    const timer = window.setInterval(() => {
      setProgress((currentProgress) => {
        if (currentProgress >= MAX_PENDING_PROGRESS) {
          return MAX_PENDING_PROGRESS
        }
        const increment = currentProgress < 70 ? 2 : 1
        return Math.min(MAX_PENDING_PROGRESS, currentProgress + increment)
      })
    }, 90)

    return () => window.clearInterval(timer)
  }, [])

  const status = useMemo(() => loadingStatus(progress, detail), [progress, detail])
  const strokeDashoffset = CIRCUMFERENCE - (progress / 100) * CIRCUMFERENCE

  return (
    <main
      className="bg-tactical relative flex min-h-[100dvh] w-full select-none items-center justify-center overflow-hidden font-mono text-slate-300"
      data-titlebar-drag-debug={titlebarDragDebugEnabled ? "true" : undefined}
    >
      <div
        aria-hidden="true"
        className="absolute inset-x-0 top-0 z-20 h-14"
        data-tauri-drag-region={tauriDragRegion}
        data-titlebar-drag-region="loading-titlebar"
        onPointerDownCapture={(event) => {
          if (isTauri && shouldStartWindowDrag(event)) {
            void startWindowDrag()
          }
        }}
      />
      <section
        aria-label="Dashboard startup"
        aria-live="polite"
        className="relative z-10 flex flex-col items-center gap-8 px-6 text-center"
      >
        <div className="relative flex items-center justify-center">
          <svg
            aria-hidden="true"
            className="-rotate-90 transform"
            height="176"
            viewBox="0 0 160 160"
            width="176"
          >
            <circle
              className="fill-none stroke-tactical/10"
              cx="80"
              cy="80"
              r={RADIUS}
              strokeWidth="3"
            />
            <circle
              className="loading-progress-ring fill-none stroke-tactical"
              cx="80"
              cy="80"
              r={RADIUS}
              strokeDasharray={CIRCUMFERENCE}
              strokeDashoffset={strokeDashoffset}
              strokeLinecap="square"
              strokeWidth="3"
            />
          </svg>

          <div className="absolute flex flex-col items-center justify-center">
            {progress >= MAX_PENDING_PROGRESS ? (
              <EyeOff aria-hidden="true" className="loading-core-icon size-8 text-tactical" />
            ) : (
              <span className="loading-progress-value text-3xl font-bold text-tactical">
                {progress}
                <span className="ml-0.5 text-sm text-tactical/70">%</span>
              </span>
            )}
          </div>
        </div>

        <div className="flex flex-col items-center gap-3">
          <div className="flex items-center gap-2">
            <Loader2 aria-hidden="true" className="size-3 animate-spin text-tactical" />
            <h1 className="loading-status tactical-overline text-tactical">{status}</h1>
          </div>

          <p className="max-w-72 text-[11px] leading-4 text-text-muted">{detail}</p>
        </div>
      </section>
    </main>
  )
}

async function startWindowDrag(): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window")
  await getCurrentWindow().startDragging()
}

function shouldStartWindowDrag(event: React.PointerEvent<HTMLElement>): boolean {
  return event.button === 0
}

function readTitlebarDragDebugFlag(): boolean {
  if (typeof window === "undefined") {
    return false
  }
  return new URLSearchParams(window.location.search).get("debug_titlebar_drag") === "1"
}

function loadingStatus(progress: number, detail: string): string {
  if (progress < 20) {
    return "RESOLVING JOURNAL DIR..."
  }
  if (progress < 50) {
    return "PARSING FLIGHT JOURNAL..."
  }
  if (progress < 85) {
    return "ESTABLISHING MATRIX RELAY..."
  }
  if (progress < MAX_PENDING_PROGRESS) {
    return "SYNCING COMMANDER DATA..."
  }
  if (detail.trim().length > 0) {
    return "AWAITING TELEMETRY LINK..."
  }
  return "AWAITING DASHBOARD SNAPSHOT..."
}
