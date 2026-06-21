import { AlertTriangle } from "lucide-react"
import type { AppSnapshot, DashboardConnectionState, EventFeedItem } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { lineSafeText } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

type WarningRailProps = {
  readonly snapshot: AppSnapshot
  readonly connection: DashboardConnectionState
}

export function WarningRail({ snapshot, connection }: WarningRailProps): React.JSX.Element {
  const warnings = warningItems(snapshot, connection)

  return (
    <Card role="region" aria-label="Warning rail" className="overflow-hidden rounded-md">
      <PanelHeader
        icon={<AlertTriangle aria-hidden="true" />}
        title="Warning rail"
        description={warnings.length === 0 ? "No active warning events." : "Attention queue"}
      />
      <ol className="divide-y">
        {warnings.length === 0 ? (
          <li className="min-h-[var(--feed-row-min-height)] p-4 text-sm text-muted-foreground">
            No active warnings.
          </li>
        ) : (
          warnings.map((warning) => (
            <li
              key={warning.id}
              className="grid min-h-[var(--feed-row-min-height)] grid-cols-[auto_1fr] gap-3 p-3"
            >
              <Badge variant={warning.level >= 3 ? "destructive" : "secondary"}>
                {warning.level >= 3 ? "Critical" : "Warning"}
              </Badge>
              <p className="min-w-0 break-words text-sm text-muted-foreground">
                {lineSafeText(warning.summary)}
              </p>
            </li>
          ))
        )}
      </ol>
    </Card>
  )
}

function warningItems(
  snapshot: AppSnapshot,
  connection: DashboardConnectionState,
): readonly EventFeedItem[] {
  const eventWarnings = snapshot.event_feed.filter((event) => event.level >= 2)
  if (connection.status !== "degraded" && connection.status !== "error") {
    return eventWarnings
  }

  return [
    {
      id: `connection:${connection.status}`,
      source: "web",
      event_type: "connection",
      level: connection.status === "error" ? 3 : 2,
      summary: connection.detail,
      timestamp: snapshot.generated_at,
      timestamp_display: snapshot.generated_at_display,
    },
    ...eventWarnings,
  ]
}
