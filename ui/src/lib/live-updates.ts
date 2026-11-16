/**
 * Flash class for a value that just changed. Two classes with identical
 * keyframes alternate so that consecutive changes restart the animation.
 */
export type FlashClass = "" | "fx-a" | "fx-b"

export function nextFlash(current: FlashClass): FlashClass {
  return current === "fx-a" ? "fx-b" : "fx-a"
}

type FeedItem = { readonly id: string }

/** Ids of events that were not in the previous feed, newest first. */
export function freshEventIds(
  previousIds: ReadonlySet<string>,
  events: readonly FeedItem[],
): ReadonlySet<string> {
  return new Set(events.filter((event) => !previousIds.has(event.id)).map((event) => event.id))
}

/** Identity of a feed: its event ids in order. */
export function feedSignature(events: readonly FeedItem[]): string {
  return events.map((event) => event.id).join("\n")
}
