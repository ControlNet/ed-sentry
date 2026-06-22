import { Activity } from "lucide-react"
import type { EventFeedItem } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import { normalizeEventFeed } from "@/store/snapshot-normalization"
import { eventTitle, lineSafeText } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

export function EventFeed({
  events,
}: {
  readonly events: readonly EventFeedItem[]
}): React.JSX.Element {
  const visibleEvents = normalizeEventFeed(events)

  return (
    <Card role="region" aria-label="Recent event feed" className="overflow-hidden rounded-md">
      <PanelHeader
        icon={<Activity aria-hidden="true" />}
        title="Recent event feed"
        description="Current-process events from the adapter stream."
      />
      <ol className="max-h-[32rem] divide-y overflow-y-auto overscroll-contain">
        {visibleEvents.length === 0 ? (
          <li className="min-h-[var(--feed-row-min-height)] p-4 text-sm text-muted-foreground">
            No dashboard events have arrived.
          </li>
        ) : (
          visibleEvents.map((event) => (
            <li
              key={event.id}
              className="grid min-h-[var(--feed-row-min-height)] grid-cols-[5.25rem_1fr] gap-3 p-3"
            >
              <time className="font-mono text-xs text-muted-foreground">
                {event.timestamp_display}
              </time>
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <p className="font-medium">{eventTitle(event)}</p>
                  <EventBadge level={event.level} />
                </div>
                <p className="mt-1 break-words text-sm text-muted-foreground">
                  {lineSafeText(event.summary)}
                </p>
              </div>
            </li>
          ))
        )}
      </ol>
    </Card>
  )
}

function EventBadge({ level }: { readonly level: number }): React.JSX.Element {
  if (level >= 3) {
    return <Badge variant="destructive">Critical</Badge>
  }
  if (level >= 2) {
    return <Badge variant="secondary">Warning</Badge>
  }
  return <Badge variant="outline">Info</Badge>
}
