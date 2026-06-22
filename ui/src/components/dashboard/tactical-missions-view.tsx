import { Activity, List } from "lucide-react"
import { useMemo, useState } from "react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { MissionDetail } from "./tactical-mission-detail"
import { MissionListButton } from "./tactical-mission-list"
import { TacticalBadge, TacticalPanel } from "./tactical-ui"

export function TacticalMissionsView({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  const [selectedId, setSelectedId] = useState<number | null>(
    snapshot.missions.items[0]?.mission_id ?? null,
  )
  const selectedMission = useMemo(
    () =>
      snapshot.missions.items.find((mission) => mission.mission_id === selectedId) ??
      snapshot.missions.items[0] ??
      null,
    [selectedId, snapshot.missions.items],
  )

  return (
    <div className="tactical-workspace grid min-h-0 grid-cols-[minmax(16rem,0.42fr)_minmax(0,1fr)] gap-4 animate-in fade-in duration-500">
      <TacticalPanel
        title="Mission Directory"
        icon={List}
        className="min-w-0"
        rightElement={<TacticalBadge>{snapshot.missions.status_label}</TacticalBadge>}
      >
        <div className="space-y-2 pr-1">
          {snapshot.missions.items.length === 0 ? (
            <p className="py-8 text-center font-mono text-xs uppercase text-slate-600">
              No active missions
            </p>
          ) : (
            snapshot.missions.items.map((mission) => (
              <MissionListButton
                key={mission.mission_id}
                mission={mission}
                selected={mission.mission_id === selectedMission?.mission_id}
                onSelect={() => setSelectedId(mission.mission_id)}
              />
            ))
          )}
        </div>
      </TacticalPanel>

      <TacticalPanel title="Mission Intel" icon={Activity} className="min-w-0">
        {selectedMission === null ? (
          <div className="flex h-full items-center justify-center font-mono text-xs uppercase tracking-widest text-slate-600">
            Awaiting selection
          </div>
        ) : (
          <MissionDetail mission={selectedMission} />
        )}
      </TacticalPanel>
    </div>
  )
}
