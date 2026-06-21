import { ClipboardList } from "lucide-react"
import type { MissionView } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { missionProgressLabel, missionProgressPercent } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

export function MissionPanel({
  missions,
  status,
}: {
  readonly missions: readonly MissionView[]
  readonly status: string
}): React.JSX.Element {
  return (
    <Card role="region" aria-label="Mission progress" className="overflow-hidden rounded-md">
      <PanelHeader
        icon={<ClipboardList aria-hidden="true" />}
        title="Mission progress"
        description={status}
      />
      <div className="overflow-x-auto">
        {missions.length === 0 ? (
          <p className="p-4 text-sm text-muted-foreground">No tracked missions in this snapshot.</p>
        ) : (
          <table className="w-full min-w-[38rem] table-fixed border-collapse">
            <thead className="bg-muted text-left text-xs text-muted-foreground">
              <tr>
                <th scope="col" className="w-[34%] px-4 py-2 font-medium">
                  Mission
                </th>
                <th scope="col" className="w-[22%] px-3 py-2 font-medium">
                  Faction
                </th>
                <th scope="col" className="w-[28%] px-3 py-2 font-medium">
                  Progress
                </th>
                <th scope="col" className="w-[16%] px-4 py-2 text-right font-medium">
                  State
                </th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {missions.map((mission) => (
                <MissionRow key={mission.mission_id} mission={mission} />
              ))}
            </tbody>
          </table>
        )}
      </div>
    </Card>
  )
}

function MissionRow({ mission }: { readonly mission: MissionView }): React.JSX.Element {
  const progressPercent = missionProgressPercent(mission.progress)
  return (
    <tr className="h-[var(--table-row-height)]">
      <td className="px-4 py-3 align-middle">
        <p className="truncate font-semibold tracking-normal">{mission.display_name}</p>
        <p className="mt-1 truncate text-xs text-muted-foreground">{mission.kind_label}</p>
      </td>
      <td className="px-3 py-3 align-middle text-sm text-muted-foreground">
        <p className="truncate">{mission.issuing_faction ?? "Unknown faction"}</p>
      </td>
      <td className="px-3 py-3 align-middle">
        <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <span className="truncate">{missionProgressLabel(mission.progress)}</span>
          <span className="shrink-0 font-mono">{mission.reward.display}</span>
        </div>
        <div className="mt-2 h-1.5 rounded-sm bg-muted">
          <div
            className="h-1.5 rounded-sm bg-data-mission"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      </td>
      <td className="px-4 py-3 text-right align-middle">
        <Badge variant="outline">{mission.state_label}</Badge>
      </td>
    </tr>
  )
}
