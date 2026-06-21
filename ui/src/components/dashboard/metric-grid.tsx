import type { SessionView } from "@/adapters/dashboard"
import { Card } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { assertNever } from "./dashboard-helpers"

type MetricTile = {
  readonly label: string
  readonly value: string
  readonly detail: string
  readonly tone: "scan" | "kill" | "mission" | "status" | "neutral"
}

export function MetricGrid({ session }: { readonly session: SessionView }): React.JSX.Element {
  return (
    <section aria-label="Combat metrics" className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
      {metricTiles(session).map((metric) => (
        <MetricCard key={metric.label} metric={metric} />
      ))}
    </section>
  )
}

function MetricCard({ metric }: { readonly metric: MetricTile }): React.JSX.Element {
  return (
    <Card className="min-w-[var(--metric-min-width)] rounded-md p-4">
      <div className="flex items-start justify-between gap-3">
        <p className="text-sm font-medium text-muted-foreground">{metric.label}</p>
        <span className={cn("h-2 w-8 rounded-sm", metricToneClass(metric.tone))} />
      </div>
      <p className="mt-2 font-mono text-2xl font-semibold tracking-normal">{metric.value}</p>
      <p className="mt-1 text-sm text-muted-foreground">{metric.detail}</p>
    </Card>
  )
}

function metricTiles(session: SessionView): readonly MetricTile[] {
  return [
    {
      label: "Kills",
      value: String(session.kills),
      detail: `${session.kill_recent_rate_per_hour.display} recent`,
      tone: "kill",
    },
    {
      label: "Cargo scans",
      value: String(session.scans),
      detail: `${session.scan_recent_rate_per_hour.display} recent`,
      tone: "scan",
    },
    {
      label: "Bounties",
      value: session.bounty_total.display,
      detail: `${session.merits_to_report} merits pending`,
      tone: "mission",
    },
    {
      label: "Ship state",
      value: session.ship_hull_display,
      detail: `Shields ${session.shields_display}; fighter ${session.fighter_hull_display}`,
      tone: session.shields_up === false ? "status" : "neutral",
    },
  ]
}

function metricToneClass(tone: MetricTile["tone"]): string {
  switch (tone) {
    case "scan":
      return "bg-data-scan"
    case "kill":
      return "bg-data-kill"
    case "mission":
      return "bg-data-mission"
    case "status":
      return "bg-status-warning"
    case "neutral":
      return "bg-status-neutral"
    default:
      return assertNever(tone)
  }
}
