import clsx from "clsx"
import type { ServiceStatusKind } from "@/adapters/dashboard"
import type { Tone } from "@/lib/events"
import { segmentsOn } from "@/lib/telemetry"

export type { Tone }

/** Full-window frame with the ambient backdrop shared by every screen. */
export function AppFrame({
  children,
  dragDebug,
}: {
  readonly children: React.ReactNode
  readonly dragDebug?: boolean
}): React.JSX.Element {
  return (
    <div className="app m-full" data-titlebar-drag-debug={dragDebug === true ? "true" : undefined}>
      <div className="stars" aria-hidden="true" />
      <div className="twinkle" aria-hidden="true" />
      <div className="stars2" aria-hidden="true" />
      <div className="plane" aria-hidden="true" />
      {children}
    </div>
  )
}

export function Panel({
  label,
  title,
  right,
  className,
  children,
  live,
}: {
  readonly label: string
  readonly title: React.ReactNode
  readonly right?: React.ReactNode
  readonly className?: string
  readonly children: React.ReactNode
  readonly live?: "off" | "polite"
}): React.JSX.Element {
  return (
    <section className={clsx("pnl", className)} aria-label={label} aria-live={live}>
      <h2 className="pt">
        <span>{title}</span>
        {right}
      </h2>
      {children}
    </section>
  )
}

/** Status text with a square mark: filled when on, outlined when off. */
export function Ind({
  tone,
  children,
  live = false,
  className,
}: {
  readonly tone: Tone
  readonly children: React.ReactNode
  readonly live?: boolean
  readonly className?: string
}): React.JSX.Element {
  return (
    <span className={clsx("ind", `t-${tone}`, className)}>
      <span className={tone === "off" ? "sqo" : clsx("sq", live && "live")} />
      {children}
    </span>
  )
}

export function Segs({
  fraction,
  label,
  count = 25,
}: {
  readonly fraction: number | null | undefined
  readonly label: string
  readonly count?: number
}): React.JSX.Element {
  const on = segmentsOn(fraction, count)
  return (
    <div className="segs" role="img" aria-label={label}>
      {Array.from({ length: count }, (_, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: fixed-length decorative segments
        <i key={index} className={index < on ? "on" : undefined} />
      ))}
    </div>
  )
}

export function Bar({ percent }: { readonly percent: number }): React.JSX.Element {
  return (
    <div className="bar">
      <div className="barf" style={{ width: `${Math.min(100, Math.max(0, percent))}%` }} />
    </div>
  )
}

export function serviceTone(kind: ServiceStatusKind): Tone {
  switch (kind) {
    case "running":
      return "ok"
    case "warning":
    case "starting":
      return "warn"
    case "error":
      return "bad"
    case "disabled":
      return "off"
    default:
      return assertNever(kind)
  }
}

/** Stroke icon paths (24x24) for service rows. */
export const iconPaths = {
  journal: "M3 5a9 3 0 1 0 18 0a9 3 0 1 0-18 0M3 5v14a9 3 0 0 0 18 0V5M3 12a9 3 0 0 0 18 0",
  matrix: "M22 12h-4l-3 9L9 3l-3 9H2",
  web: "M12 20h.01M2 8.82a15 15 0 0 1 20 0M5 12.86a10 10 0 0 1 14 0M8.5 16.43a5 5 0 0 1 7 0",
  tunnel:
    "M4.9 16.1C1 12.2 1 5.8 4.9 1.9M7.8 4.7a6.14 6.14 0 0 0-.8 7.5M16.2 4.8c2 2 2.26 5.11.8 7.47M19.1 1.9a9.96 9.96 0 0 1 0 14.1M8 22l4-11 4 11M10 9a2 2 0 1 0 4 0a2 2 0 1 0-4 0",
} as const

export function StrokeIcon({
  path,
  className,
  style,
}: {
  readonly path: string
  readonly className?: string
  readonly style?: React.CSSProperties
}): React.JSX.Element {
  return (
    <svg className={clsx("ico", className)} viewBox="0 0 24 24" aria-hidden="true" style={style}>
      <path d={path} />
    </svg>
  )
}

function assertNever(value: never): never {
  throw new Error(`Unhandled service status: ${String(value)}`)
}
