import { Activity, AlertTriangle } from "lucide-react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { lineSafeText } from "./dashboard-helpers"
import { MissionSummaryRow } from "./tactical-telemetry-widgets"
import { ProgressBar, TacticalBadge, TacticalPanel } from "./tactical-ui"

type MissionSummary = AppSnapshot["missions"]["items"][number]

type SharedKillProgress = {
  readonly completed: number
  readonly total: number
  readonly remaining: number
}

type IssuerKillGroup = {
  readonly required: number
  readonly remaining: number
}

export function TacticalMissionSummary({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  const killProgress = sharedKillProgress(snapshot.missions.items)
  return (
    <TacticalPanel
      title="Active Missions"
      icon={Activity}
      ariaLabel="Mission progress"
      className="tactical-summary-panel md:col-span-2"
      rightElement={
        <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
          <TacticalBadge>
            <span data-testid="telemetry-active-mission-count">
              ACTIVE {snapshot.missions.status_label}
            </span>
          </TacticalBadge>
          <MissionHeaderChip
            label="Kills"
            value={`${killProgress.completed}/${killProgress.total}`}
            testId="telemetry-mission-kill-progress"
          >
            <ProgressBar
              current={killProgress.completed}
              total={killProgress.total}
              className="mt-0 h-1 w-8 shrink-0"
              tone="mission"
            />
          </MissionHeaderChip>
          <MissionHeaderChip
            label="ETA"
            value={missionEtaLabel(
              killProgress.remaining,
              snapshot.session.kill_total_rate_per_hour.value,
            )}
            testId="telemetry-mission-kill-eta"
          />
        </div>
      }
    >
      <div className="space-y-2">
        {snapshot.missions.items.length === 0 ? (
          <p className="py-8 text-center font-mono text-xs uppercase text-slate-600">
            No active missions
          </p>
        ) : (
          snapshot.missions.items.map((mission) => (
            <MissionSummaryRow key={mission.mission_id} mission={mission} />
          ))
        )}
      </div>
    </TacticalPanel>
  )
}

function MissionHeaderChip({
  label,
  value,
  testId,
  children,
}: {
  readonly label: string
  readonly value: string
  readonly testId: string
  readonly children?: React.ReactNode
}): React.JSX.Element {
  return (
    <div
      className="inline-flex items-center gap-1 border border-slate-700 bg-slate-900/80 px-2 py-0.5 font-mono text-[9px] font-bold uppercase tracking-wider text-slate-400"
      data-testid={testId}
      title={`${label}: ${value}`}
    >
      <span>{label}</span>
      <span className="text-slate-300">{value}</span>
      {children}
    </div>
  )
}

function sharedKillProgress(missions: readonly MissionSummary[]): SharedKillProgress {
  const stacks = new Map<string, Map<string, IssuerKillGroup>>()

  for (const mission of missions) {
    if (mission.progress.kind !== "massacre") {
      continue
    }

    const stackKey = mission.progress.target_faction ?? `mission:${mission.mission_id}`
    const issuerKey = mission.issuing_faction ?? `mission:${mission.mission_id}`
    const issuerGroups = stacks.get(stackKey) ?? new Map<string, IssuerKillGroup>()
    const current = issuerGroups.get(issuerKey) ?? { required: 0, remaining: 0 }
    const remaining = Math.max(0, mission.progress.kill_count - mission.progress.kills)

    issuerGroups.set(issuerKey, {
      required: current.required + mission.progress.kill_count,
      remaining: current.remaining + remaining,
    })
    stacks.set(stackKey, issuerGroups)
  }

  let total = 0
  let remaining = 0
  for (const issuerGroups of stacks.values()) {
    total += maxIssuerValue(issuerGroups, "required")
    remaining += maxIssuerValue(issuerGroups, "remaining")
  }

  return {
    completed: Math.max(0, total - remaining),
    total,
    remaining,
  }
}

function maxIssuerValue(
  issuerGroups: ReadonlyMap<string, IssuerKillGroup>,
  field: keyof IssuerKillGroup,
): number {
  let value = 0
  for (const group of issuerGroups.values()) {
    value = Math.max(value, group[field])
  }
  return value
}

function missionEtaLabel(remainingKills: number, killRatePerHour: number): string {
  if (remainingKills <= 0) {
    return "Complete"
  }
  if (killRatePerHour <= 0) {
    return "-"
  }

  const remainingMinutes = Math.max(1, Math.round((remainingKills / killRatePerHour) * 60))
  if (remainingMinutes < 60) {
    return `${remainingMinutes}m`
  }

  const hours = Math.floor(remainingMinutes / 60)
  const minutes = remainingMinutes % 60
  if (minutes === 0) {
    return `${hours}h`
  }
  return `${hours}h ${minutes}m`
}

export function TacticalRecentAlerts({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  return (
    <TacticalPanel
      title="Recent Alerts"
      icon={AlertTriangle}
      ariaLabel="Recent event feed"
      className="tactical-summary-panel md:col-span-2"
    >
      <div className="space-y-2 font-mono text-[10px]">
        {snapshot.event_feed.length === 0 ? (
          <p className="py-8 text-center font-mono text-xs uppercase text-slate-600">
            No dashboard events have arrived.
          </p>
        ) : (
          snapshot.event_feed.map((event) => (
            <div
              key={event.id}
              data-testid="telemetry-event-row"
              className="grid grid-cols-[5.5rem_minmax(0,1fr)] gap-3 border-l-2 border-slate-800 bg-slate-900/20 py-1.5 pl-2"
            >
              <span className="text-slate-600">{event.timestamp_display}</span>
              <span className="truncate text-slate-300">
                [{event.source.toUpperCase()}] {lineSafeText(event.summary)}
              </span>
            </div>
          ))
        )}
      </div>
    </TacticalPanel>
  )
}
