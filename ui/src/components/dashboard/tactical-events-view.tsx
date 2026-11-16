import clsx from "clsx"
import { useState } from "react"
import type { AppSnapshot, EventFeedItem } from "@/adapters/dashboard"
import {
  type EventCategory,
  eventCategory,
  eventCategoryCounts,
  eventCategoryLabels,
  eventLevelTone,
  eventTypeLabel,
  presentEventCategories,
} from "@/lib/events"
import { parseTimestamp, utcClock } from "@/lib/format"
import { sessionTimelineMarks } from "@/lib/timeline"
import { lineSafeText } from "./dashboard-helpers"
import { Ind, Panel } from "./g1-ui"
import { useFreshEventIds } from "./use-live-updates"

type Filter = "all" | EventCategory

export function TacticalEventsView({
  snapshot,
  nowMs,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
}): React.JSX.Element {
  const [filter, setFilter] = useState<Filter>("all")
  const events = snapshot.event_feed
  const freshIds = useFreshEventIds(events)
  const categories = presentEventCategories(events)
  const counts = eventCategoryCounts(events)
  const activeFilter: Filter = filter === "all" || categories.includes(filter) ? filter : "all"
  const shown =
    activeFilter === "all" ? events : events.filter((item) => eventCategory(item) === activeFilter)

  return (
    <div className="col" style={{ gap: 16 }}>
      {events.length === 0 ? null : (
        <SessionTimeline
          snapshot={snapshot}
          nowMs={nowMs}
          categories={categories}
          counts={counts}
          filter={activeFilter}
          onFilter={setFilter}
        />
      )}
      <Panel
        label="Event log"
        title="Event log"
        right={<span className="lab">{shown.length} shown</span>}
      >
        {shown.length === 0 ? (
          <p className="sec2" style={{ margin: 0 }}>
            No dashboard events have arrived.
          </p>
        ) : (
          <div className="list">
            {shown.map((item, index) => (
              <EventEntry
                key={item.id}
                item={item}
                newest={index === 0}
                fresh={freshIds.has(item.id)}
              />
            ))}
          </div>
        )}
      </Panel>
    </div>
  )
}

function SessionTimeline({
  snapshot,
  nowMs,
  categories,
  counts,
  filter,
  onFilter,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
  readonly categories: readonly EventCategory[]
  readonly counts: ReadonlyMap<EventCategory, number>
  readonly filter: Filter
  readonly onFilter: (filter: Filter) => void
}): React.JSX.Element {
  const events = snapshot.event_feed
  const times = events.flatMap((item) => {
    const atMs = parseTimestamp(item.timestamp)
    return atMs === null ? [] : [atMs]
  })
  const startMs = Math.min(
    parseTimestamp(snapshot.session.started_at) ?? Number.POSITIVE_INFINITY,
    ...times,
  )
  const endMs = Math.max(nowMs, ...times)
  const span = Math.max(1, endMs - startMs)
  const marks = sessionTimelineMarks(startMs, endMs)
  const chips: readonly { readonly id: Filter; readonly label: string; readonly count: number }[] =
    [
      { id: "all", label: "All", count: events.length },
      ...categories.map((category) => ({
        id: category,
        label: eventCategoryLabels[category],
        count: counts.get(category) ?? 0,
      })),
    ]

  return (
    <Panel label="Session timeline" title="Session timeline">
      <fieldset className="chips" aria-label="Filter by type">
        {chips.map((chip) => (
          <button
            key={chip.id}
            type="button"
            className={clsx("tchip", filter === chip.id && "tchip-on")}
            aria-pressed={filter === chip.id}
            onClick={() => onFilter(chip.id)}
          >
            <span>{chip.label}</span>
            <span className="c">{chip.count}</span>
          </button>
        ))}
      </fieldset>
      <div className="lanes">
        {categories.map((category) => {
          const list = events.filter((item) => eventCategory(item) === category)
          const dim = filter !== "all" && filter !== category ? "lane-dim" : undefined
          const latest = list[0]
          const latestMs = parseTimestamp(latest?.timestamp)
          return (
            <LaneRow
              key={category}
              category={category}
              dim={dim}
              items={list}
              startMs={startMs}
              span={span}
              last={latestMs === null ? "—" : utcClock(latestMs)}
            />
          )
        })}
        <span />
        <span className="axis stl-axis" style={{ height: 18, lineHeight: "18px" }}>
          {marks.map((mark, index) => (
            <span
              key={mark.label}
              className={index === 0 ? undefined : index === marks.length - 1 ? "ax-r cy" : "ax-c"}
              style={{ left: `${mark.left}%` }}
            >
              {mark.label}
            </span>
          ))}
        </span>
        <span className="lane-t cy" style={{ fontSize: 12, letterSpacing: ".08em" }}>
          NOW
        </span>
      </div>
    </Panel>
  )
}

function LaneRow({
  category,
  dim,
  items,
  startMs,
  span,
  last,
}: {
  readonly category: EventCategory
  readonly dim: string | undefined
  readonly items: readonly EventFeedItem[]
  readonly startMs: number
  readonly span: number
  readonly last: string
}): React.JSX.Element {
  return (
    <>
      <span className={clsx("lane-l", dim)}>
        <i className={clsx("sq", `ty-${category}`)} style={{ width: 8, height: 8 }} />
        {eventCategoryLabels[category]}
      </span>
      <span className={clsx("lane", dim)} data-testid={`event-lane-${category}`}>
        {items.map((item) => {
          const atMs = parseTimestamp(item.timestamp)
          if (atMs === null) {
            return null
          }
          return (
            <i
              key={item.id}
              className={`ty-${category}`}
              style={{ left: `${((atMs - startMs) / span) * 100}%` }}
              title={`${utcClock(atMs)} ${eventTypeLabel(item)}`}
            />
          )
        })}
        <i className="nowm" style={{ left: "100%" }} />
      </span>
      <span className={clsx("lane-t", dim)}>{last}</span>
    </>
  )
}

function EventEntry({
  item,
  newest,
  fresh,
}: {
  readonly item: EventFeedItem
  readonly newest: boolean
  readonly fresh: boolean
}): React.JSX.Element {
  const atMs = parseTimestamp(item.timestamp)
  return (
    <article className={clsx("entry ev", fresh && "fresh")} data-testid="event-log-entry">
      <div>
        <p style={{ margin: 0, fontSize: 16, fontWeight: 500 }}>
          {atMs === null ? item.timestamp_display : utcClock(atMs)}
        </p>
      </div>
      <div style={{ minWidth: 0 }}>
        <Ind tone={eventLevelTone(item.level)}>{eventTypeLabel(item)}</Ind>
        <p style={{ margin: "4px 0 0", fontSize: 16, color: "#e8ecf1", overflowWrap: "anywhere" }}>
          {lineSafeText(item.summary)}
          {newest ? <span className="cur" aria-hidden="true" /> : null}
        </p>
      </div>
      <div style={{ textAlign: "right" }}>
        <p className="k" style={{ margin: 0 }}>
          {item.source}
        </p>
        <p className="sec2" style={{ margin: "2px 0 0", fontSize: 13 }}>
          Level {item.level}
        </p>
      </div>
    </article>
  )
}
