import { useState } from "react"
import { type FlashClass, feedSignature, freshEventIds, nextFlash } from "@/lib/live-updates"

/**
 * Flash class for a displayed value: empty on mount, then alternating
 * "fx-a" / "fx-b" each time the value changes. A remounted view starts
 * empty again, so switching tabs never replays an old flash.
 */
export function useChangeFlash(value: string | number): FlashClass {
  const [state, setState] = useState<{ value: string | number; flash: FlashClass }>(() => ({
    value,
    flash: "",
  }))
  if (state.value === value) {
    return state.flash
  }
  const next = { value, flash: nextFlash(state.flash) }
  setState(next)
  return next.flash
}

type FeedState = {
  readonly signature: string
  readonly ids: ReadonlySet<string>
  readonly fresh: ReadonlySet<string>
}

const noFreshEvents: ReadonlySet<string> = new Set()

/**
 * Ids of the events that arrived with the latest feed change. The feed shown
 * when the view mounts has no fresh events; a later snapshot that keeps the
 * same feed keeps the same set, so the highlight never restarts.
 */
export function useFreshEventIds(events: readonly { readonly id: string }[]): ReadonlySet<string> {
  const signature = feedSignature(events)
  const [state, setState] = useState<FeedState>(() => ({
    signature,
    ids: new Set(events.map((event) => event.id)),
    fresh: noFreshEvents,
  }))
  if (state.signature === signature) {
    return state.fresh
  }
  const next: FeedState = {
    signature,
    ids: new Set(events.map((event) => event.id)),
    fresh: freshEventIds(state.ids, events),
  }
  setState(next)
  return next.fresh
}
