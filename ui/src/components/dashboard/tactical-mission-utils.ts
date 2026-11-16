import type { MissionView } from "@/adapters/dashboard"
import { assertNever } from "./dashboard-helpers"

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
