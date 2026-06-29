import type { AppSnapshot, EventFeedItem } from "@/adapters/dashboard"

export function normalizeSnapshot(snapshot: AppSnapshot): AppSnapshot {
  return {
    ...snapshot,
    event_feed: normalizeEventFeed(snapshot.event_feed),
  }
}

export function shouldApplySnapshotUpdate(current: AppSnapshot, next: AppSnapshot): boolean {
  return stableSnapshotKey(current) !== stableSnapshotKey(next)
}

export function normalizeEventFeed(items: readonly EventFeedItem[]): AppSnapshot["event_feed"] {
  return [...items]
    .sort((left, right) => {
      const timestampOrder = eventTimestampMs(right) - eventTimestampMs(left)
      if (timestampOrder !== 0) {
        return timestampOrder
      }
      return right.id.localeCompare(left.id)
    })
    .slice(0, 30)
}

function stableSnapshotKey(snapshot: AppSnapshot): string {
  return JSON.stringify({
    session: snapshot.session,
    missions: snapshot.missions,
    afk_checklist: snapshot.afk_checklist,
    journal_source: snapshot.journal_source,
    matrix: snapshot.matrix,
    web: snapshot.web,
    tunnel: snapshot.tunnel,
  })
}

function eventTimestampMs(item: EventFeedItem): number {
  const timestampMs = Date.parse(item.timestamp)
  return Number.isNaN(timestampMs) ? 0 : timestampMs
}
