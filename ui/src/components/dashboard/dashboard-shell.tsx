import type { LucideIcon } from "lucide-react"
import { Activity, EyeOff, List, Minus, Settings, Square, Terminal, X } from "lucide-react"
import { useState } from "react"
import type { AppSnapshot, DashboardAdapter, DashboardConnectionState } from "@/adapters/dashboard"
import { cn } from "@/lib/utils"
import { TacticalEventsView } from "./tactical-events-view"
import { TacticalMissionsView } from "./tactical-missions-view"
import { TacticalSystemsView } from "./tactical-systems-view"
import { TacticalTelemetryView } from "./tactical-telemetry-view"

type DashboardShellProps = {
  readonly snapshot: AppSnapshot
  readonly adapter: DashboardAdapter
  readonly connection: DashboardConnectionState
  readonly isRefreshing: boolean
  readonly onRefresh: () => void
}

type WorkspaceTab = "dashboard" | "missions" | "events" | "config"

const workspaceTabs = [
  {
    id: "dashboard",
    label: "Telemetry",
    icon: Activity,
    title: "Telemetry Interface",
  },
  {
    id: "missions",
    label: "Missions",
    icon: List,
    title: "Missions Interface",
  },
  {
    id: "events",
    label: "Comms Feed",
    icon: Terminal,
    title: "Events Interface",
  },
  {
    id: "config",
    label: "Systems",
    icon: Settings,
    title: "Config Interface",
  },
] as const satisfies readonly {
  readonly id: WorkspaceTab
  readonly label: string
  readonly icon: typeof Activity
  readonly title: string
}[]

export function DashboardShell({
  snapshot,
  adapter,
  connection,
}: DashboardShellProps): React.JSX.Element {
  const [activeTab, setActiveTab] = useState<WorkspaceTab>("dashboard")
  const titlebarDragDebugEnabled = readTitlebarDragDebugFlag()
  const activeDefinition = workspaceTabs.find((tab) => tab.id === activeTab) ?? workspaceTabs[0]
  const isTauri = adapter.mode === "tauri"
  const showTauriChrome = isTauri || titlebarDragDebugEnabled
  const tauriDragRegion = showTauriChrome ? "" : undefined

  return (
    <div
      className="bg-tactical relative h-screen w-full select-none overflow-hidden bg-[#03060a] font-sans text-slate-300"
      data-titlebar-drag-debug={titlebarDragDebugEnabled ? "true" : undefined}
    >
      <div className="relative z-10 flex h-screen flex-col">
        <header
          className="z-20 flex h-14 shrink-0 items-center justify-between gap-3 border-b border-orange-500/20 bg-[#04070d]/90 pl-4 shadow-lg backdrop-blur-md md:pl-6"
          data-tauri-drag-region={tauriDragRegion}
          data-titlebar-drag-region="titlebar"
          onPointerDownCapture={(event) => {
            if (isTauri && shouldStartWindowDrag(event)) {
              void startWindowDrag()
            }
          }}
        >
          <div
            className="flex w-auto min-w-0 items-center gap-3 md:w-48"
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="brand"
          >
            <div
              className="flex size-6 shrink-0 items-center justify-center rounded-sm border border-orange-500/40 bg-orange-950/20 shadow-[inset_0_0_8px_rgba(249,115,22,0.3)]"
              data-tauri-drag-region={tauriDragRegion}
              data-titlebar-drag-region="brand-mark"
            >
              <EyeOff
                aria-hidden="true"
                className="size-3 text-tactical"
                data-tauri-drag-region={tauriDragRegion}
              />
            </div>
            <div
              className="hidden min-w-0 sm:block"
              data-tauri-drag-region={tauriDragRegion}
              data-titlebar-drag-region="brand-label"
            >
              <h1
                className="text-shadow-glow truncate text-sm font-black uppercase tracking-wider text-slate-200"
                data-tauri-drag-region={tauriDragRegion}
              >
                ed-sentry
              </h1>
              <p className="sr-only">AFK monitor</p>
            </div>
          </div>

          <nav
            aria-label="Primary"
            className="custom-scrollbar relative min-w-0 flex-1 overflow-x-auto"
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="primary-nav"
          >
            <div
              className="flex w-fit gap-2"
              data-tauri-drag-region={tauriDragRegion}
              data-titlebar-drag-region="primary-nav-list"
            >
              {workspaceTabs.map((tab) => (
                <WorkspaceTabButton
                  key={tab.id}
                  tab={tab}
                  active={activeTab === tab.id}
                  onSelect={() => setActiveTab(tab.id)}
                />
              ))}
            </div>
          </nav>

          <div
            className="flex w-auto shrink-0 items-center justify-end gap-3 md:w-48"
            data-tauri-drag-region={tauriDragRegion}
            data-titlebar-drag-region="status"
          >
            <span
              className={connectionStatusClass(connection.status)}
              data-tauri-drag-region={tauriDragRegion}
              data-titlebar-drag-region="status-label"
            >
              <span
                className="size-1.5 animate-pulse rounded-full bg-current shadow-[0_0_5px_currentColor]"
                data-tauri-drag-region={tauriDragRegion}
              />
              {connectionStatusLabel(connection.status)}
            </span>
          </div>
          {showTauriChrome ? <WindowControls /> : <div className="w-4 shrink-0 md:w-6" />}
        </header>

        <main className="custom-scrollbar relative flex min-h-0 flex-1 flex-col overflow-y-auto bg-gradient-to-b from-[#04070d]/50 to-transparent p-6">
          <div className="mb-6 flex shrink-0 items-center justify-between border-b border-orange-500/10 pb-4">
            <div>
              <h1 className="text-shadow-glow text-xl font-bold uppercase tracking-widest text-slate-200">
                {activeDefinition.title}
              </h1>
              <p className="font-mono text-[9px] uppercase text-slate-500">
                SYS_RELAY: {connection.status.toUpperCase()}
              </p>
            </div>
          </div>

          {renderWorkspace(activeTab, snapshot, adapter)}
        </main>
      </div>
    </div>
  )
}

type WindowCommand = "minimize" | "maximize" | "close"
type ClosestCapableElement = Element & {
  closest(selectors: string): Element | null
}

function WindowControls(): React.JSX.Element {
  return (
    <div
      className="flex h-full shrink-0 items-stretch border-l border-orange-500/15"
      data-titlebar-no-drag="window-controls"
    >
      <WindowControlButton command="minimize" label="Minimize window" icon={Minus} />
      <WindowControlButton command="maximize" label="Maximize window" icon={Square} />
      <WindowControlButton command="close" label="Close window" icon={X} danger />
    </div>
  )
}

function WindowControlButton({
  command,
  label,
  icon: Icon,
  danger = false,
}: {
  readonly command: WindowCommand
  readonly label: string
  readonly icon: LucideIcon
  readonly danger?: boolean
}): React.JSX.Element {
  return (
    <button
      type="button"
      aria-label={label}
      data-titlebar-no-drag="window-control"
      data-window-control={command}
      title={label}
      onClick={() => {
        void runWindowCommand(command)
      }}
      className={cn(
        "flex h-full w-12 items-center justify-center text-slate-500 transition-colors duration-150 hover:bg-slate-900/70 hover:text-slate-100 focus-visible:outline focus-visible:outline-1 focus-visible:outline-offset-[-2px] focus-visible:outline-orange-400",
        danger && "hover:bg-rose-950/80 hover:text-rose-200",
      )}
    >
      <Icon aria-hidden="true" className="size-3.5" />
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

function WorkspaceTabButton({
  tab,
  active,
  onSelect,
}: {
  readonly tab: (typeof workspaceTabs)[number]
  readonly active: boolean
  readonly onSelect: () => void
}): React.JSX.Element {
  const Icon = tab.icon
  return (
    <button
      type="button"
      onClick={onSelect}
      aria-current={active ? "page" : undefined}
      aria-label={tab.id === "config" ? "Systems Config" : tab.label}
      data-titlebar-no-drag="workspace-tab"
      className={cn(
        "flex items-center gap-2 whitespace-nowrap rounded-sm border-b-2 px-4 py-2 text-[10px] font-bold uppercase tracking-widest transition-all duration-200",
        active
          ? "border-orange-500 bg-orange-500/10 text-orange-400 shadow-[inset_0_-10px_10px_-10px_rgba(249,115,22,0.5)]"
          : "border-transparent text-slate-400 hover:bg-slate-900/60 hover:text-slate-200",
      )}
    >
      <Icon
        aria-hidden="true"
        className={cn(
          "size-3.5",
          active ? "text-orange-500 drop-shadow-[0_0_5px_rgba(249,115,22,0.8)]" : "opacity-60",
        )}
      />
      <span>{tab.label}</span>
    </button>
  )
}

function connectionStatusLabel(status: DashboardConnectionState["status"]): string {
  switch (status) {
    case "connected":
      return "SYNCED"
    case "loading":
      return "LOADING"
    case "degraded":
      return "DEGRADED"
    case "error":
      return "ERROR"
    case "idle":
      return "IDLE"
    default:
      return assertNever(status)
  }
}

function connectionStatusClass(status: DashboardConnectionState["status"]): string {
  switch (status) {
    case "connected":
      return "hidden items-center gap-1.5 font-mono text-[9px] text-emerald-500 sm:flex"
    case "loading":
      return "hidden items-center gap-1.5 font-mono text-[9px] text-amber-400 sm:flex"
    case "degraded":
      return "hidden items-center gap-1.5 font-mono text-[9px] text-amber-400 sm:flex"
    case "error":
      return "hidden items-center gap-1.5 font-mono text-[9px] text-rose-500 sm:flex"
    case "idle":
      return "hidden items-center gap-1.5 font-mono text-[9px] text-slate-500 sm:flex"
    default:
      return assertNever(status)
  }
}

function renderWorkspace(
  activeTab: WorkspaceTab,
  snapshot: AppSnapshot,
  adapter: DashboardAdapter,
): React.JSX.Element {
  switch (activeTab) {
    case "dashboard":
      return <TacticalTelemetryView snapshot={snapshot} />
    case "missions":
      return <TacticalMissionsView snapshot={snapshot} />
    case "events":
      return <TacticalEventsView snapshot={snapshot} />
    case "config":
      return <TacticalSystemsView adapter={adapter} />
    default:
      return assertNever(activeTab)
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled workspace tab: ${String(value)}`)
}
