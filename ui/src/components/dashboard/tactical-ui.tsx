import type { LucideIcon } from "lucide-react"
import { CheckCircle2, XCircle } from "lucide-react"
import type { ReactNode } from "react"
import { cn } from "@/lib/utils"

export type TacticalBadgeTone = "default" | "success" | "warning" | "danger" | "brand"

export function TacticalPanel({
  title,
  icon: Icon,
  rightElement,
  children,
  className,
  bodyClassName,
  ariaLabel,
}: {
  readonly title: string
  readonly icon?: LucideIcon
  readonly rightElement?: ReactNode
  readonly children: ReactNode
  readonly className?: string
  readonly bodyClassName?: string
  readonly ariaLabel?: string
}): React.JSX.Element {
  return (
    <section
      aria-label={ariaLabel ?? title}
      className={cn(
        "hud-panel relative flex min-h-0 flex-col overflow-hidden rounded-sm",
        className,
      )}
    >
      <CornerMarks />
      <div className="flex shrink-0 items-center justify-between border-b border-orange-500/20 bg-orange-950/20 px-3 py-2">
        <div className="flex min-w-0 items-center gap-2">
          {Icon === undefined ? null : (
            <Icon aria-hidden="true" className="size-3.5 text-tactical" />
          )}
          <h2 className="truncate text-[10px] font-bold uppercase tracking-widest text-tactical">
            {title}
          </h2>
        </div>
        {rightElement}
      </div>
      <div className={cn("custom-scrollbar min-h-0 flex-1 overflow-auto p-4", bodyClassName)}>
        {children}
      </div>
    </section>
  )
}

export function TacticalBadge({
  children,
  tone = "default",
  className,
}: {
  readonly children: ReactNode
  readonly tone?: TacticalBadgeTone
  readonly className?: string
}): React.JSX.Element {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 border px-2 py-0.5 font-mono text-[9px] font-bold uppercase tracking-wider",
        badgeToneClass(tone),
        className,
      )}
    >
      {children}
    </span>
  )
}

export function ProgressBar({
  current,
  total,
  tone = "brand",
}: {
  readonly current: number
  readonly total: number
  readonly tone?: "brand" | "success" | "danger" | "scan" | "mission"
}): React.JSX.Element {
  const percent = total <= 0 ? 0 : Math.min(100, Math.max(0, (current / total) * 100))
  return (
    <div className="mt-1 h-1.5 w-full overflow-hidden border border-slate-800 bg-slate-900/50">
      <div
        className={cn("h-full transition-all duration-500 ease-out", progressToneClass(tone))}
        style={{ width: `${percent}%` }}
      />
    </div>
  )
}

export function StatusGlyph({
  tone,
}: {
  readonly tone: "success" | "warning" | "danger" | "neutral"
}): React.JSX.Element {
  if (tone === "success") {
    return <CheckCircle2 aria-hidden="true" className="size-3.5 text-status-online" />
  }
  if (tone === "danger") {
    return <XCircle aria-hidden="true" className="size-3.5 text-status-danger" />
  }
  return (
    <span
      aria-hidden="true"
      className={cn(
        "mt-1 size-2 shrink-0 shadow-[0_0_5px_currentColor]",
        tone === "warning" ? "bg-status-warning text-status-warning" : "bg-status-neutral",
      )}
    />
  )
}

export function DataRow({
  label,
  value,
  valueClassName,
}: {
  readonly label: string
  readonly value: ReactNode
  readonly valueClassName?: string
}): React.JSX.Element {
  return (
    <div className="flex items-end justify-between gap-3 border-b border-tactical-accent/10 pb-1 font-mono text-[10px]">
      <span className="text-text-muted">{label}</span>
      <span className={cn("min-w-0 truncate text-right text-text-primary", valueClassName)}>
        {value}
      </span>
    </div>
  )
}

function CornerMarks(): React.JSX.Element {
  return (
    <>
      <span className="absolute left-0 top-0 size-2 border-l-2 border-t-2 border-tactical-accent/50" />
      <span className="absolute right-0 top-0 size-2 border-r-2 border-t-2 border-tactical-accent/50" />
      <span className="absolute bottom-0 left-0 size-2 border-b-2 border-l-2 border-tactical-accent/50" />
      <span className="absolute bottom-0 right-0 size-2 border-b-2 border-r-2 border-tactical-accent/50" />
    </>
  )
}

function badgeToneClass(tone: TacticalBadgeTone): string {
  switch (tone) {
    case "success":
      return "border-emerald-800 bg-emerald-950/80 text-emerald-400 shadow-[0_0_10px_rgba(16,185,129,0.2)]"
    case "warning":
      return "border-amber-800 bg-amber-950/80 text-amber-400 shadow-[0_0_10px_rgba(245,158,11,0.2)]"
    case "danger":
      return "border-rose-800 bg-rose-950/80 text-rose-400 shadow-[0_0_10px_rgba(225,29,72,0.2)]"
    case "brand":
      return "border-orange-800 bg-orange-950/80 text-orange-400 shadow-[0_0_10px_rgba(249,115,22,0.2)]"
    case "default":
      return "border-slate-700 bg-slate-900/80 text-slate-400"
    default:
      return assertNever(tone)
  }
}

function progressToneClass(tone: "brand" | "success" | "danger" | "scan" | "mission"): string {
  switch (tone) {
    case "brand":
      return "bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]"
    case "success":
      return "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"
    case "danger":
      return "bg-rose-500 shadow-[0_0_8px_rgba(225,29,72,0.6)]"
    case "scan":
      return "bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.6)]"
    case "mission":
      return "bg-violet-500 shadow-[0_0_8px_rgba(139,92,246,0.6)]"
    default:
      return assertNever(tone)
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled tactical variant: ${String(value)}`)
}
