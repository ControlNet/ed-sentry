import { RadioTower, Wifi } from "lucide-react"
import type { ServiceStatusView } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { lineSafeText } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

type ServiceStatusPanelProps = {
  readonly matrix: ServiceStatusView
  readonly web: ServiceStatusView
}

export function ServiceStatusPanel({ matrix, web }: ServiceStatusPanelProps): React.JSX.Element {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-1">
      <ServicePanel
        ariaLabel="Matrix status"
        icon={<RadioTower aria-hidden="true" />}
        title="Matrix status"
        status={matrix}
        fallback="No token exposed"
      />
      <ServicePanel
        ariaLabel="Web status"
        icon={<Wifi aria-hidden="true" />}
        title="Web status"
        status={web}
        fallback="No Web API detail"
      />
    </div>
  )
}

function ServicePanel({
  ariaLabel,
  icon,
  title,
  status,
  fallback,
}: {
  readonly ariaLabel: string
  readonly icon: React.ReactElement
  readonly title: string
  readonly status: ServiceStatusView
  readonly fallback: string
}): React.JSX.Element {
  return (
    <Card role="region" aria-label={ariaLabel} className="overflow-hidden rounded-md">
      <PanelHeader icon={icon} title={title} description={status.status_label} />
      <div className="grid gap-3 p-4">
        <div className="flex min-h-[var(--feed-row-min-height)] items-center justify-between gap-3 rounded-md border bg-background p-3">
          <div className="min-w-0">
            <p className="text-sm font-medium">{status.status_label}</p>
            <p className="mt-1 break-words font-mono text-xs text-muted-foreground">
              {serviceDetail(status, fallback)}
            </p>
          </div>
          <Badge variant={status.kind === "error" ? "destructive" : "outline"}>{status.kind}</Badge>
        </div>
      </div>
    </Card>
  )
}

function serviceDetail(status: ServiceStatusView, fallback: string): string {
  return lineSafeText(
    status.message ?? status.room_id ?? status.bind_address ?? status.url ?? fallback,
  )
}
