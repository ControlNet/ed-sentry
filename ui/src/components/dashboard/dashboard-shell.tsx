import clsx from "clsx"
import { useRef, useState } from "react"
import type { AppSnapshot, DashboardAdapter, DashboardConnectionState } from "@/adapters/dashboard"
import { formatMinutes, parseTimestamp, utcClock } from "@/lib/format"
import { AppFrame, Ind, type Tone } from "./g1-ui"
import { NavCardArt } from "./nav-card-art"
import { TacticalAboutView } from "./tactical-about-view"
import { TacticalEventsView } from "./tactical-events-view"
import { TacticalMissionsView } from "./tactical-missions-view"
import { TacticalSystemsView } from "./tactical-systems-view"
import { TacticalTelemetryView } from "./tactical-telemetry-view"
import { useLiveNow } from "./use-live-now"
import { useChangeFlash } from "./use-live-updates"

type DashboardShellProps = {
  readonly snapshot: AppSnapshot
  readonly adapter: DashboardAdapter
  readonly connection: DashboardConnectionState
  readonly isRefreshing: boolean
  readonly onRefresh: () => void
}

type WorkspaceTab = "dashboard" | "missions" | "events" | "config" | "about"

const workspaceTabs = [
  { id: "dashboard", label: "Dashboard" },
  { id: "missions", label: "Missions" },
  { id: "events", label: "Events" },
  { id: "config", label: "Systems" },
  { id: "about", label: "About" },
] as const satisfies readonly { readonly id: WorkspaceTab; readonly label: string }[]

export function DashboardShell({
  snapshot,
  adapter,
  connection,
}: DashboardShellProps): React.JSX.Element {
  const [activeTab, setActiveTab] = useState<WorkspaceTab>("dashboard")
  const scrollRef = useRef<HTMLElement | null>(null)
  const nowMs = useLiveNow(snapshot.generated_at)
  const titlebarDragDebugEnabled = readTitlebarDragDebugFlag()
  const activeDefinition = workspaceTabs.find((tab) => tab.id === activeTab) ?? workspaceTabs[0]
  const isTauri = adapter.mode === "tauri"
  const showTauriChrome = isTauri || titlebarDragDebugEnabled
  const tauriDragRegion = showTauriChrome ? "" : undefined
  const system = snapshot.session.system ?? null
  const lastEventMs = parseTimestamp(snapshot.event_feed[0]?.timestamp)
  const lastEventFlash = useChangeFlash(lastEventMs ?? "")

  const selectTab = (tab: WorkspaceTab): void => {
    setActiveTab(tab)
    if (scrollRef.current !== null) {
      scrollRef.current.scrollTop = 0
    }
  }

  return (
    <AppFrame dragDebug={titlebarDragDebugEnabled}>
      <header
        className="hdr layer"
        data-tauri-drag-region={tauriDragRegion}
        data-titlebar-drag-region="titlebar"
        onPointerDownCapture={(event) => {
          if (isTauri && shouldStartWindowDrag(event)) {
            void startWindowDrag()
          }
        }}
      >
        <div
          className="brand"
          data-tauri-drag-region={tauriDragRegion}
          data-titlebar-drag-region="brand"
        >
          <img
            src="/logo.png"
            alt=""
            width="34"
            height="34"
            style={{ flexShrink: 0, objectFit: "contain" }}
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="brand-mark"
          />
          <div
            style={{ minWidth: 0 }}
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="brand-label"
          >
            <p className="hctx" data-tauri-drag-region={tauriDragRegion}>
              <span className="or">ED Sentry</span>
              {system === null ? null : (
                <>
                  <span className="pipe">|</span>
                  <span className="sec2">{system}</span>
                </>
              )}
            </p>
            <h1 className="htl" data-tauri-drag-region={tauriDragRegion}>
              {activeDefinition.label}
            </h1>
          </div>
        </div>
        <div
          style={{ display: "flex", alignItems: "center", gap: 20 }}
          data-tauri-drag-region={tauriDragRegion}
          data-titlebar-drag-region="status"
        >
          <span
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="status-label"
            data-connection-status={connection.status}
            title={connection.detail}
          >
            <ConnectionIndicator status={connection.status} />
          </span>
          <div className="clock" data-tauri-drag-region={tauriDragRegion}>
            <span className="lab">Last event</span>
            <b className={lastEventFlash}>{lastEventMs === null ? "—" : utcClock(lastEventMs)}</b>
          </div>
        </div>
        {showTauriChrome ? <WindowControls /> : null}
      </header>

      <div className="body layer">
        <nav className="side" aria-label="Primary">
          {workspaceTabs.map((tab) => (
            <button
              key={tab.id}
              type="button"
              className={clsx("card", activeTab === tab.id && "card-on")}
              aria-label={tab.label}
              aria-current={activeTab === tab.id ? "page" : undefined}
              data-workspace-tab={tab.id}
              onClick={() => selectTab(tab.id)}
            >
              <NavCardArt kind={tab.id} />
              <span className="capb">
                <span className="t">{tab.label}</span>
              </span>
            </button>
          ))}
        </nav>

        <div className="main">
          <main className="scroll" ref={scrollRef}>
            {renderWorkspace(activeTab, snapshot, adapter, nowMs, () => selectTab("events"))}
          </main>
          <footer className="keys">
            <span className="sb">
              Session <span className="wh">{sessionElapsed(snapshot, nowMs)}</span>
            </span>
            <CommanderLine session={snapshot.session} />
          </footer>
        </div>
      </div>
    </AppFrame>
  )
}

function ConnectionIndicator({
  status,
}: {
  readonly status: DashboardConnectionState["status"]
}): React.JSX.Element {
  if (status === "connected") {
    return (
      <span className="sync">
        <span className="sq" />
        Synced
      </span>
    )
  }
  const [tone, label] = connectionTone(status)
  return <Ind tone={tone}>{label}</Ind>
}

function connectionTone(
  status: Exclude<DashboardConnectionState["status"], "connected">,
): readonly [Tone, string] {
  switch (status) {
    case "loading":
      return ["warn", "Loading"]
    case "degraded":
      return ["warn", "Degraded"]
    case "error":
      return ["bad", "Error"]
    case "idle":
      return ["off", "Idle"]
    default:
      return assertNever(status)
  }
}

function sessionElapsed(snapshot: AppSnapshot, nowMs: number): string {
  const startedMs = parseTimestamp(snapshot.session.started_at)
  if (!snapshot.session.active || startedMs === null) {
    return snapshot.session.elapsed_display
  }
  return formatMinutes((nowMs - startedMs) / 1000)
}

function CommanderLine({
  session,
}: {
  readonly session: AppSnapshot["session"]
}): React.JSX.Element | null {
  const commander = session.commander ?? null
  const details = [session.ship, session.mode].filter(
    (value): value is string => typeof value === "string" && value !== "",
  )
  if (commander === null && details.length === 0) {
    return null
  }
  return (
    <span className="sb cmdr">
      {commander === null ? null : <span className="wh">CMDR {commander}</span>}
      {details.map((detail, index) => (
        <span key={detail}>
          {index > 0 || commander !== null ? " · " : ""}
          {detail}
        </span>
      ))}
    </span>
  )
}

type WindowCommand = "minimize" | "maximize" | "close"
type ClosestCapableElement = Element & {
  closest(selectors: string): Element | null
}

const windowControlIcons: Readonly<Record<WindowCommand, string>> = {
  minimize: "M5 12h14",
  maximize: "M5 5h14v14H5z",
  close: "M18 6 6 18M6 6l12 12",
}

function WindowControls(): React.JSX.Element {
  return (
    <div
      className="winctl"
      style={{ display: "flex", gap: 2 }}
      data-titlebar-no-drag="window-controls"
    >
      <WindowControlButton command="minimize" label="Minimize window" />
      <WindowControlButton command="maximize" label="Maximize window" />
      <WindowControlButton command="close" label="Close window" />
    </div>
  )
}

function WindowControlButton({
  command,
  label,
}: {
  readonly command: WindowCommand
  readonly label: string
}): React.JSX.Element {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={clsx("winb", command === "close" && "winx")}
      data-titlebar-no-drag="window-control"
      data-window-control={command}
      onClick={() => {
        void runWindowCommand(command)
      }}
    >
      <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
        <path d={windowControlIcons[command]} />
      </svg>
    </button>
  )
}

async function runWindowCommand(command: WindowCommand): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window")
  const currentWindow = getCurrentWindow()
  switch (command) {
    case "minimize":
      await currentWindow.minimize()
      return
    case "maximize":
      await currentWindow.toggleMaximize()
      return
    case "close":
      await currentWindow.close()
      return
    default:
      return assertNever(command)
  }
}

async function startWindowDrag(): Promise<void> {
  const { getCurrentWindow } = await import("@tauri-apps/api/window")
  await getCurrentWindow().startDragging()
}

function shouldStartWindowDrag(event: React.PointerEvent<HTMLElement>): boolean {
  if (event.button !== 0) {
    return false
  }
  if (!(event.target instanceof Element)) {
    return false
  }
  return !isInteractiveTitlebarTarget(event.target)
}

function isInteractiveTitlebarTarget(target: ClosestCapableElement): boolean {
  return (
    target.closest(
      "button, a, input, textarea, select, summary, [role='button'], [role='tab'], [data-window-control], [data-titlebar-no-drag]",
    ) !== null
  )
}

function readTitlebarDragDebugFlag(): boolean {
  if (typeof window === "undefined") {
    return false
  }
  return new URLSearchParams(window.location.search).get("debug_titlebar_drag") === "1"
}

function renderWorkspace(
  activeTab: WorkspaceTab,
  snapshot: AppSnapshot,
  adapter: DashboardAdapter,
  nowMs: number,
  openEvents: () => void,
): React.JSX.Element {
  switch (activeTab) {
    case "dashboard":
      return <TacticalTelemetryView snapshot={snapshot} nowMs={nowMs} onOpenEvents={openEvents} />
    case "missions":
      return <TacticalMissionsView snapshot={snapshot} nowMs={nowMs} />
    case "events":
      return <TacticalEventsView snapshot={snapshot} nowMs={nowMs} />
    case "config":
      return <TacticalSystemsView adapter={adapter} />
    case "about":
      return <TacticalAboutView />
    default:
      return assertNever(activeTab)
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled shell variant: ${String(value)}`)
}
