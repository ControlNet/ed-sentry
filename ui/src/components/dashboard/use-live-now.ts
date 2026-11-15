import { useEffect, useState } from "react"
import { parseTimestamp } from "@/lib/format"
import { liveNowMs } from "@/lib/telemetry"

const TICK_MS = 1_000

type Anchor = {
  readonly generatedMs: number
  readonly receivedMs: number
}

/**
 * Current time on the snapshot clock, advancing every second between
 * snapshots. Live counters (since last kill, session time) use it so they
 * stay correct even when the local clock differs from the monitor's clock.
 */
export function useLiveNow(generatedAt: string): number {
  const generatedMs = parseTimestamp(generatedAt) ?? 0
  const [anchor, setAnchor] = useState<Anchor>(() => ({ generatedMs, receivedMs: Date.now() }))
  const [currentMs, setCurrentMs] = useState(() => Date.now())

  if (anchor.generatedMs !== generatedMs) {
    setAnchor({ generatedMs, receivedMs: Date.now() })
  }

  useEffect(() => {
    const timer = globalThis.setInterval(() => setCurrentMs(Date.now()), TICK_MS)
    return () => globalThis.clearInterval(timer)
  }, [])

  return liveNowMs(anchor.generatedMs, anchor.receivedMs, currentMs)
}
