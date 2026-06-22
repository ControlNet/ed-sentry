import { Activity, AlertTriangle } from "lucide-react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { lineSafeText } from "./dashboard-helpers"
import { MissionSummaryRow } from "./tactical-telemetry-widgets"
import { TacticalBadge, TacticalPanel } from "./tactical-ui"

export function TacticalMissionSummary({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  return (
    <TacticalPanel
      title="Active Missions"
      icon={Activity}
      ariaLabel="Mission progress"
      className="tactical-summary-panel md:col-span-2"
      rightElement={<TacticalBadge>TOTAL {snapshot.missions.total_count}</TacticalBadge>}
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
