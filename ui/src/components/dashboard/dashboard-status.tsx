import { AlertTriangle, CircleDot, RefreshCw, Wifi } from "lucide-react"
import type { ConnectionStatus, ServiceStatusKind } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { assertNever, statusTextClass } from "./dashboard-helpers"

type PanelHeaderProps = {
  readonly icon: React.ReactElement
  readonly title: string
  readonly description: string
}

export function PanelHeader({ icon, title, description }: PanelHeaderProps): React.JSX.Element {
  return (
    <div className="border-b p-4">
      <div className="flex items-center gap-2">
        <span className="text-primary">{icon}</span>
        <h2 className="font-semibold tracking-normal">{title}</h2>
      </div>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

type StatusLineProps = {
  readonly icon: React.ReactElement
  readonly label: string
  readonly value: string
  readonly detail: string
  readonly kind?: ServiceStatusKind
}

export function StatusLine({
  icon,
  label,
  value,
  detail,
  kind = "disabled",
}: StatusLineProps): React.JSX.Element {
  return (
    <div className="rounded-md border bg-background p-3">
      <div className="flex items-center gap-2 text-sm font-medium">
        <span className={statusTextClass(kind)}>{icon}</span>
        <span>{label}</span>
      </div>
      <p className="mt-2 text-sm font-semibold">{value}</p>
      <p className="mt-1 break-words font-mono text-xs text-muted-foreground">{detail}</p>
    </div>
  )
}

export function StatusPill({
  kind,
  label,
}: {
  readonly kind: ServiceStatusKind
  readonly label: string
}): React.JSX.Element {
  return (
    <Badge variant={kind === "error" ? "destructive" : kind === "running" ? "default" : "outline"}>
      {label}
    </Badge>
  )
}

export function ConnectionIcon({
  status,
}: {
  readonly status: ConnectionStatus
}): React.JSX.Element {
  switch (status) {
    case "connected":
      return <Wifi aria-hidden="true" className="mt-0.5 size-5 text-status-online" />
    case "loading":
      return <RefreshCw aria-hidden="true" className="mt-0.5 size-5 animate-spin text-primary" />
    case "degraded":
      return <AlertTriangle aria-hidden="true" className="mt-0.5 size-5 text-status-warning" />
    case "error":
      return <AlertTriangle aria-hidden="true" className="mt-0.5 size-5 text-status-danger" />
    case "idle":
      return <CircleDot aria-hidden="true" className="mt-0.5 size-5 text-status-neutral" />
    default:
      return assertNever(status)
  }
}
