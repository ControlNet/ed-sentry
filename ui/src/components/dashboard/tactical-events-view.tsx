import { AlertTriangle, CheckCircle2, Terminal, XCircle } from "lucide-react"
import type { AppSnapshot, EventFeedItem } from "@/adapters/dashboard"
import { eventTitle, lineSafeText } from "./dashboard-helpers"
import { TacticalBadge, TacticalPanel } from "./tactical-ui"

export function TacticalEventsView({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  return (
    <TacticalPanel
      title="System Telemetry Feed"
      icon={Terminal}
      className="tactical-workspace animate-in fade-in duration-500"
    >
      <div className="sticky -top-4 z-10 -mx-4 mb-2 grid grid-cols-12 gap-4 border-b border-orange-500/20 bg-[#060a11]/95 px-4 pb-2 pt-4 font-mono text-[9px] font-bold uppercase tracking-widest text-slate-600 shadow-[0_5px_15px_-5px_rgba(0,0,0,0.5)] backdrop-blur-md">
        <div className="col-span-3 sm:col-span-2">Timestamp</div>
        <div className="col-span-3 sm:col-span-2">Origin</div>
        <div className="col-span-6 sm:col-span-8">Payload decode</div>
      </div>
      <div className="pb-4">
        {snapshot.event_feed.length === 0 ? (
          <p className="p-6 text-center font-mono text-xs uppercase tracking-widest text-slate-600">
            No dashboard events have arrived.
          </p>
        ) : (
          snapshot.event_feed.map((event) => <EventRow key={event.id} event={event} />)
        )}
      </div>
    </TacticalPanel>
  )
}

function EventRow({ event }: { readonly event: EventFeedItem }): React.JSX.Element {
  return (
    <div className="grid grid-cols-12 gap-4 border-b border-slate-800/30 p-2 font-mono text-[10px] transition-colors hover:bg-slate-900/60">
      <div className="col-span-3 pt-0.5 text-slate-500 sm:col-span-2">
        {event.timestamp_display}
      </div>
      <div className="col-span-3 sm:col-span-2">
        <TacticalBadge tone={event.source === "matrix" ? "brand" : "default"}>
          {event.source}
        </TacticalBadge>
      </div>
      <div className="col-span-6 min-w-0 sm:col-span-8">
        <p className={`mb-0.5 flex items-center gap-2 font-bold ${eventToneClass(event.level)}`}>
          {eventIcon(event.level)}
          {eventTitle(event).toUpperCase()}
        </p>
        <p className="break-words text-slate-500">{lineSafeText(event.summary)}</p>
      </div>
    </div>
  )
}

function eventIcon(level: number): React.JSX.Element | null {
  if (level >= 3) {
    return <XCircle aria-hidden="true" className="size-3" />
  }
  if (level >= 2) {
    return <AlertTriangle aria-hidden="true" className="size-3" />
  }
  if (level === 1) {
    return <CheckCircle2 aria-hidden="true" className="size-3" />
  }
  return null
}

function eventToneClass(level: number): string {
  if (level >= 3) {
    return "text-rose-500"
  }
  if (level >= 2) {
    return "text-amber-400"
  }
  if (level === 1) {
    return "text-emerald-400"
  }
  return "text-slate-300"
}
