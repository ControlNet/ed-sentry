import { Gauge } from "lucide-react"
import type { EventFeedItem, SessionView } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { fuelSummary, healthPercent, healthToneClass } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

type HealthPanelProps = {
  readonly session: SessionView
  readonly events: readonly EventFeedItem[]
}

export function HealthPanel({ session, events }: HealthPanelProps): React.JSX.Element {
  return (
    <Card role="region" aria-label="Health and fuel" className="overflow-hidden rounded-md">
      <PanelHeader
        icon={<Gauge aria-hidden="true" />}
        title="Health and fuel"
        description="Hull, shields, fighter, and last fuel report."
      />
      <div className="grid gap-3 p-4">
        <HealthMeter
          label="Ship hull"
          value={session.ship_hull_display}
          percent={session.ship_hull_percent}
        />
        <HealthMeter
          label="Fighter hull"
          value={session.fighter_hull_display}
          percent={session.fighter_hull_percent}
        />
        <div className="flex min-h-[var(--feed-row-min-height)] items-center justify-between gap-3 rounded-md border bg-background p-3">
          <div className="min-w-0">
            <p className="text-sm font-medium">Shields</p>
            <p className="truncate text-sm text-muted-foreground">{session.shields_display}</p>
          </div>
          <Badge variant={session.shields_up === false ? "secondary" : "outline"}>
            {session.shields_up === false ? "Attention" : "Nominal"}
          </Badge>
        </div>
        <div className="min-h-[var(--feed-row-min-height)] rounded-md border bg-background p-3">
          <p className="text-sm font-medium">Fuel</p>
          <p className="mt-1 break-words text-sm text-muted-foreground">{fuelSummary(events)}</p>
        </div>
      </div>
    </Card>
  )
}

function HealthMeter({
  label,
  value,
  percent,
}: {
  readonly label: string
  readonly value: string
  readonly percent: number | null | undefined
}): React.JSX.Element {
  const width = healthPercent(percent)

  return (
    <div className="min-h-[var(--feed-row-min-height)] rounded-md border bg-background p-3">
      <div className="flex items-center justify-between gap-3 text-sm">
        <span className="font-medium">{label}</span>
        <span className="font-mono text-muted-foreground">{value}</span>
      </div>
      <div className="mt-2 h-1.5 rounded-sm bg-muted">
        <div
          className={cn("h-1.5 rounded-sm", healthToneClass(percent))}
          style={{ width: `${width}%` }}
        />
      </div>
    </div>
  )
}
