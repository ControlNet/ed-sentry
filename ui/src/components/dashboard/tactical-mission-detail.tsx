import { Activity } from "lucide-react"
import type { MissionView } from "@/adapters/dashboard"
import { assertNever } from "./dashboard-helpers"
import { destinationLabel, detailToneClass, type MissionDetailTone } from "./tactical-mission-utils"
import { ProgressBar, TacticalBadge } from "./tactical-ui"

export function MissionDetail({ mission }: { readonly mission: MissionView }): React.JSX.Element {
  return (
    <div className="h-full min-h-0">
      <div className="mb-6 border-b border-tactical-accent/20 pb-4">
        <div className="mb-3 flex flex-wrap gap-2">
          <TacticalBadge tone="brand">{mission.kind_label}</TacticalBadge>
          <TacticalBadge tone={mission.state === "active" ? "success" : "default"}>
            {mission.state_label}
          </TacticalBadge>
        </div>
        <h2 className="text-shadow-glow text-2xl font-black uppercase tracking-wider text-slate-200">
          {mission.display_name}
        </h2>
        <p className="mt-1 font-mono text-[9px] uppercase tracking-widest text-orange-500/60">
          ID: {mission.mission_id}
        </p>
      </div>

      <div className="grid gap-8 font-mono text-[10px] xl:grid-cols-2">
        <div className="space-y-6">
          <DetailSection title="Faction & Destination">
            <DetailRow label="Issuing" value={mission.issuing_faction ?? "Unknown"} />
            <DetailRow label="Target" value={mission.target_faction ?? "Unknown"} tone="danger" />
            <DetailRow label="Destination" value={destinationLabel(mission)} />
          </DetailSection>
          <DetailSection title="Timetable & Reward">
            <DetailRow label="Accepted" value={mission.accepted_at_display} />
            <DetailRow label="Expiry" value={mission.expiry_display ?? "Unknown"} tone="warning" />
            <DetailRow label="Payout" value={mission.reward.display} tone="scan" strong />
          </DetailSection>
        </div>

        <div className="relative rounded-sm border border-slate-800 bg-slate-900/50 p-4">
          <div className="absolute left-0 top-0 size-8 border-l border-t border-orange-500/30 opacity-50" />
          <h3 className="mb-4 flex items-center gap-2 font-bold uppercase tracking-widest text-orange-500">
            <Activity aria-hidden="true" className="size-3" />
            Tracking uplink
          </h3>
          <MissionProgressDetail mission={mission} />
        </div>
      </div>
    </div>
  )
}

function MissionProgressDetail({ mission }: { readonly mission: MissionView }): React.JSX.Element {
  switch (mission.progress.kind) {
    case "massacre": {
      const remaining = Math.max(0, mission.progress.kill_count - mission.progress.kills)
      return (
        <div className="space-y-3">
          <DetailRow
            label="Confirmed kills"
            value={`${mission.progress.kills} / ${mission.progress.kill_count}`}
            strong
          />
          <DetailRow label="Remaining" value={String(remaining)} tone="warning" />
          <DetailRow label="Target faction" value={mission.progress.target_faction ?? "Unknown"} />
          <ProgressBar current={mission.progress.kills} total={mission.progress.kill_count} />
          <p className="mt-4 text-[9px] leading-relaxed text-slate-500">
            Target lock requires valid bounty progress for the tracked faction.
          </p>
        </div>
      )
    }
    case "trade": {
      const remainingToCollect = Math.max(0, mission.progress.count - mission.progress.collected)
      const remainingToDeliver = Math.max(0, mission.progress.count - mission.progress.delivered)
      return (
        <div className="space-y-3">
          <DetailRow
            label="Commodity"
            value={mission.progress.commodity ?? "Unknown"}
            tone="brand"
            strong
          />
          <DetailRow label="Required units" value={String(mission.progress.count)} />
          <DetailRow label="Collected" value={String(mission.progress.collected)} tone="scan" />
          <DetailRow label="Delivered" value={String(mission.progress.delivered)} tone="success" />
          <DetailRow label="Remaining collect" value={String(remainingToCollect)} />
          <DetailRow label="Remaining deliver" value={String(remainingToDeliver)} tone="warning" />
          <ProgressBar
            current={mission.progress.delivered}
            total={mission.progress.count}
            tone="success"
          />
          <div className="pt-2">
            <TacticalBadge>Cargo Depot progress</TacticalBadge>
          </div>
        </div>
      )
    }
    case "none":
      return (
        <p className="font-mono text-[10px] font-bold uppercase tracking-widest text-slate-500">
          No structured progress available.
        </p>
      )
    default:
      return assertNever(mission.progress)
  }
}

function DetailSection({
  title,
  children,
}: {
  readonly title: string
  readonly children: React.ReactNode
}): React.JSX.Element {
  return (
    <section>
      <h3 className="mb-2 border-b border-slate-800 pb-1 font-bold uppercase tracking-widest text-orange-500">
        {title}
      </h3>
      <div className="space-y-1">{children}</div>
    </section>
  )
}

function DetailRow({
  label,
  value,
  tone = "default",
  strong = false,
}: {
  readonly label: string
  readonly value: string
  readonly tone?: MissionDetailTone
  readonly strong?: boolean
}): React.JSX.Element {
  return (
    <div className="flex justify-between gap-4">
      <span className="text-slate-600">{label.toUpperCase()}</span>
      <span className={detailToneClass(tone, strong)}>{value}</span>
    </div>
  )
}
