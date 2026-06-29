import { Loader2, RadioTower } from "lucide-react"
import { useRef, useState } from "react"
import type { TunnelStatusKind, TunnelStatusView } from "@/adapters/dashboard"
import { cn } from "@/lib/utils"
import { lineSafeText } from "./dashboard-helpers"
import { TacticalBadge, type TacticalBadgeTone } from "./tactical-ui"
import { TunnelLinkQr } from "./tunnel-link-qr"

type TunnelServiceLineProps = {
  readonly tunnel: TunnelStatusView
  readonly onStart: () => Promise<void>
}

export function TunnelServiceLine({ tunnel, onStart }: TunnelServiceLineProps): React.JSX.Element {
  const [qrVisible, setQrVisible] = useState(false)
  const [starting, setStarting] = useState(false)
  const tunnelLinkRef = useRef<HTMLAnchorElement | null>(null)
  const canStart = tunnelCanStart(tunnel)
  const publicUrl = tunnel.kind === "running" ? (tunnel.public_url ?? null) : null

  const start = async (): Promise<void> => {
    if (!canStart || starting) {
      return
    }
    setStarting(true)
    try {
      await onStart()
    } finally {
      setStarting(false)
    }
  }

  return (
    <div
      className="flex items-start justify-between gap-3"
      data-service-node="Tunnel"
      data-status-kind={tunnel.kind}
    >
      <div className="min-w-0">
        <p className="flex items-center gap-1.5 font-mono text-[10px] uppercase text-slate-500">
          <RadioTower aria-hidden="true" className="size-3 text-slate-400" />
          Tunnel
        </p>
        <div className="relative mt-1 w-fit max-w-full">
          {publicUrl === null ? (
            <p className="truncate font-mono text-[8px] text-slate-600">{tunnelDetail(tunnel)}</p>
          ) : (
            <a
              ref={tunnelLinkRef}
              className="block truncate font-mono text-[8px] text-data-scan underline-offset-2 transition-colors hover:text-tactical hover:underline focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              href={publicUrl}
              onBlur={() => setQrVisible(false)}
              onFocus={() => setQrVisible(true)}
              onPointerEnter={() => setQrVisible(true)}
              onPointerLeave={() => setQrVisible(false)}
            >
              {tunnelLinkLabel(publicUrl)}
            </a>
          )}
          {publicUrl !== null && qrVisible ? (
            <TunnelLinkQr anchor={tunnelLinkRef.current} url={publicUrl} />
          ) : null}
        </div>
      </div>
      <div className="flex shrink-0 items-center">
        {canStart ? (
          <button
            type="button"
            className="m-0 border-0 bg-transparent p-0 font-inherit text-inherit focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-wait"
            disabled={starting}
            onClick={() => {
              void start()
            }}
          >
            <TacticalBadge
              className={cn(starting ? "cursor-wait" : "cursor-pointer")}
              tone={tunnelBadgeTone(tunnel.kind)}
            >
              {starting ? <Loader2 aria-hidden="true" className="size-3 animate-spin" /> : null}
              {starting ? "STARTING" : tunnelStatusLabel(tunnel.kind)}
            </TacticalBadge>
          </button>
        ) : (
          <TacticalBadge tone={tunnelBadgeTone(tunnel.kind)}>
            {tunnelStatusLabel(tunnel.kind)}
          </TacticalBadge>
        )}
      </div>
    </div>
  )
}

function tunnelCanStart(tunnel: TunnelStatusView): boolean {
  return tunnel.kind === "start" || (tunnel.kind === "error" && tunnel.retryable_error)
}

function tunnelStatusLabel(kind: TunnelStatusKind): string {
  switch (kind) {
    case "start":
      return "START"
    case "starting":
      return "STARTING"
    case "running":
      return "RUNNING"
    case "error":
      return "ERROR"
    case "unsupported":
      return "UNSUPPORTED"
    case "disabled":
      return "UNAVAILABLE"
    default:
      return assertNever(kind)
  }
}

function tunnelBadgeTone(kind: TunnelStatusKind): TacticalBadgeTone {
  switch (kind) {
    case "running":
      return "success"
    case "start":
    case "starting":
      return "warning"
    case "error":
      return "danger"
    case "disabled":
    case "unsupported":
      return "default"
    default:
      return assertNever(kind)
  }
}

function tunnelDetail(tunnel: TunnelStatusView): string {
  if (tunnel.kind === "disabled") {
    return "Tunnel unavailable"
  }
  return lineSafeText(tunnel.message ?? tunnel.status_label)
}

function tunnelLinkLabel(url: string): string {
  try {
    return new URL(url).host
  } catch (error) {
    if (error instanceof TypeError) {
      return lineSafeText(url)
    }
    throw error
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled tunnel status: ${String(value)}`)
}
