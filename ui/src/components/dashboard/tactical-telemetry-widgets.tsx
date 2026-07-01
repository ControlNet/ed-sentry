import type { Activity } from "lucide-react"
import type { AppSnapshot, ServiceStatusKind } from "@/adapters/dashboard"
import {
  displaySafeText,
  lineSafeText,
  serviceStatusBadgeTone,
  sourceDetail,
} from "./dashboard-helpers"
import { handleExternalLinkClick } from "./external-link"
import { ProgressBar, TacticalBadge } from "./tactical-ui"

export function Meter({
  label,
  value,
  percent,
  tone = "brand",
}: {
  readonly label: string
  readonly value: string
  readonly percent: number | null | undefined
  readonly tone?: "brand" | "scan"
}): React.JSX.Element {
  const normalized =
    percent === null || percent === undefined ? 0 : percent > 1 ? percent : percent * 100
  return (
    <div>
      <div className="mb-1 flex justify-between font-mono text-[10px]">
        <span className="uppercase text-slate-500">{label}</span>
        <span className={normalized < 40 ? "animate-pulse text-rose-400" : "text-slate-200"}>
          {value}
        </span>
      </div>
      <ProgressBar current={normalized} total={100} tone={normalized < 40 ? "danger" : tone} />
    </div>
  )
}

export function MetricValue({
  label,
  value,
}: {
  readonly label: string
  readonly value: string
}): React.JSX.Element {
  return (
    <div className="border border-slate-800 bg-slate-900/40 p-2 text-center">
      <p className="text-shadow-glow font-mono text-2xl font-bold text-slate-200">{value}</p>
      <p className="mt-1 text-[9px] font-bold uppercase tracking-widest text-orange-500/80">
        {label}
      </p>
    </div>
  )
}

export function ServiceLine({
  icon: Icon,
  label,
  detail,
  detailHref,
  redactJournalFileName = true,
  badge,
  statusKind,
}: {
  readonly icon: typeof Activity
  readonly label: string
  readonly detail: string
  readonly detailHref?: string | null | undefined
  readonly redactJournalFileName?: boolean
  readonly badge: string
  readonly statusKind: ServiceStatusKind
}): React.JSX.Element {
  const detailText = redactJournalFileName ? lineSafeText(detail) : displaySafeText(detail)
  return (
    <div
      className="flex items-start justify-between gap-3"
      data-service-node={label}
      data-status-kind={statusKind}
    >
      <div className="min-w-0">
        <p className="flex items-center gap-1.5 font-mono text-[10px] uppercase text-slate-500">
          <Icon aria-hidden="true" className="size-3 text-slate-400" />
          {label}
        </p>
        <p className="mt-1 truncate font-mono text-[8px] text-slate-600">
          {detailHref === null || detailHref === undefined ? (
            detailText
          ) : (
            <a
              className="text-data-scan underline-offset-2 transition-colors hover:text-orange-300 hover:underline focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-orange-400"
              href={detailHref}
              target="_blank"
              rel="noopener noreferrer"
              onClick={handleExternalLinkClick}
            >
              {detailText}
            </a>
          )}
        </p>
      </div>
      <TacticalBadge tone={serviceStatusBadgeTone(statusKind)}>{badge}</TacticalBadge>
    </div>
  )
}

export function JournalServiceLine({
  snapshot,
  icon,
}: {
  readonly snapshot: AppSnapshot
  readonly icon: typeof Activity
}): React.JSX.Element {
  return (
    <ServiceLine
      icon={icon}
      label="Local Journal"
      detail={sourceDetail(snapshot.journal_source.folder, snapshot.journal_source.selected_file)}
      redactJournalFileName={false}
      badge={snapshot.journal_source.status_label}
      statusKind={journalStatusKind(snapshot.journal_source.status_label)}
    />
  )
}

function journalStatusKind(statusLabel: string): ServiceStatusKind {
  const normalized = statusLabel.trim().toLowerCase()
  if (normalized === "watching" || normalized === "running" || normalized === "active") {
    return "running"
  }
  if (normalized === "disabled") {
    return "disabled"
  }
  if (normalized === "warning" || normalized === "degraded" || normalized === "starting") {
    return "warning"
  }
  if (normalized === "error" || normalized === "missing") {
    return "error"
  }
  return "disabled"
}

export function MissionSummaryRow({
  mission,
}: {
  readonly mission: AppSnapshot["missions"]["items"][number]
}): React.JSX.Element {
  return (
    <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 rounded-sm border border-slate-800/50 bg-slate-900/40 p-2">
      <div className="min-w-0">
        <p className="truncate text-xs font-medium text-slate-200">{mission.display_name}</p>
        <p className="font-mono text-[9px] uppercase tracking-widest text-orange-500/70">
          {mission.kind_label}
        </p>
      </div>
      <div className="w-24 text-right font-mono text-[10px] text-slate-400">
        <p>{mission.progress.kind === "none" ? mission.state_label : mission.progress.display}</p>
        {mission.progress.kind === "none" ? null : (
          <ProgressBar
            current={missionProgressCurrent(mission)}
            total={missionProgressTotal(mission)}
            tone="brand"
          />
        )}
      </div>
    </div>
  )
}

function missionProgressCurrent(mission: AppSnapshot["missions"]["items"][number]): number {
  switch (mission.progress.kind) {
    case "massacre":
      return mission.progress.kills
    case "trade":
      return mission.progress.delivered
    case "none":
      return 0
    default:
      return assertNever(mission.progress)
  }
}

function missionProgressTotal(mission: AppSnapshot["missions"]["items"][number]): number {
  switch (mission.progress.kind) {
    case "massacre":
      return mission.progress.kill_count
    case "trade":
      return mission.progress.count
    case "none":
      return 0
    default:
      return assertNever(mission.progress)
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled telemetry mission progress: ${String(value)}`)
}
