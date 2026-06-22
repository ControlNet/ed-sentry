import type { MissionView } from "@/adapters/dashboard"
import { missionProgressCurrent, missionProgressTotal } from "./tactical-mission-utils"
import { ProgressBar } from "./tactical-ui"

export function MissionListButton({
  mission,
  selected,
  onSelect,
}: {
  readonly mission: MissionView
  readonly selected: boolean
  readonly onSelect: () => void
}): React.JSX.Element {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={
        selected
          ? "w-full rounded-sm border border-orange-500 bg-orange-950/30 p-3 text-left shadow-[inset_0_0_15px_rgba(249,115,22,0.15)] transition-all"
          : "w-full rounded-sm border border-slate-800 bg-slate-900/30 p-3 text-left transition-all hover:border-slate-600"
      }
    >
      <div className="flex items-start justify-between gap-2">
        <p className="min-w-0 truncate pr-2 text-xs font-bold text-slate-200">
          {mission.display_name}
        </p>
        <span
          aria-hidden="true"
          className={
            mission.state === "active"
              ? "mt-1 size-2 shrink-0 bg-emerald-500 shadow-[0_0_5px_#10b981]"
              : "mt-1 size-2 shrink-0 bg-slate-600"
          }
        />
      </div>
      <div className="mt-2 flex justify-between gap-3 font-mono text-[9px] uppercase tracking-widest text-slate-500">
        <span>{mission.kind_label}</span>
        <span>
          {mission.progress.kind === "none" ? mission.state_label : mission.progress.display}
        </span>
      </div>
      {mission.progress.kind === "none" ? null : (
        <ProgressBar
          current={missionProgressCurrent(mission)}
          total={missionProgressTotal(mission)}
        />
      )}
    </button>
  )
}
