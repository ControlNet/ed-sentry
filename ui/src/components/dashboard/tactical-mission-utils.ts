import type { MissionView } from "@/adapters/dashboard"
import { assertNever } from "./dashboard-helpers"

export type MissionDetailTone = "default" | "warning" | "danger" | "scan" | "success" | "brand"

export function destinationLabel(mission: MissionView): string {
  if (mission.destination_system === null && mission.destination_station === null) {
    return "Unknown"
  }
  return [mission.destination_system, mission.destination_station].filter(Boolean).join(" / ")
}

export function missionProgressCurrent(mission: MissionView): number {
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

export function missionProgressTotal(mission: MissionView): number {
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

export function detailToneClass(tone: MissionDetailTone, strong: boolean): string {
  const weight = strong ? "font-bold text-xs" : "text-[10px]"
  switch (tone) {
    case "warning":
      return `${weight} text-amber-400`
    case "danger":
      return `${weight} text-rose-400`
    case "scan":
      return `${weight} text-cyan-400`
    case "success":
      return `${weight} text-emerald-400`
    case "brand":
      return `${weight} text-orange-400`
    case "default":
      return `${weight} text-slate-300`
    default:
      return assertNever(tone)
  }
}
