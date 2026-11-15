import { useRef, useState } from "react"
import type { TunnelStatusKind, TunnelStatusView } from "@/adapters/dashboard"
import { lineSafeText } from "./dashboard-helpers"
import { handleExternalLinkClick } from "./external-link"
import { Ind, iconPaths, StrokeIcon, type Tone } from "./g1-ui"
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
  const busy = starting || tunnel.kind === "starting"

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
    <div className="row svc" data-service-node="Tunnel" data-status-kind={tunnel.kind}>
      <span style={{ display: "flex", gap: 10, minWidth: 0 }}>
        <StrokeIcon
          path={iconPaths.tunnel}
          className="or"
          style={{ width: 17, height: 17, marginTop: 3 }}
        />
        <span style={{ minWidth: 0, position: "relative" }}>
          <span className="nm">Tunnel</span>
          {publicUrl === null ? (
            <span className="dt" title={tunnelDetail(tunnel)}>
              {tunnelDetail(tunnel)}
            </span>
          ) : (
            <a
              ref={tunnelLinkRef}
              className="dt"
              style={{ color: "#3cb8ff" }}
              href={publicUrl}
              target="_blank"
              rel="noopener noreferrer"
              onClick={handleExternalLinkClick}
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
        </span>
      </span>
      {busy ? (
        <button type="button" className="btn btn-busy" aria-busy="true" disabled>
          Starting
          <span className="tsegs" aria-hidden="true">
            <i />
            <i />
            <i />
            <i />
            <i />
          </span>
        </button>
      ) : canStart ? (
        <button
          type="button"
          className="btn"
          onClick={() => {
            void start()
          }}
        >
          {tunnel.kind === "error" ? "Retry" : "Start"}
        </button>
      ) : (
        <Ind tone={tunnelTone(tunnel.kind)} live={tunnel.kind === "running"}>
          {tunnelStatusLabel(tunnel.kind)}
        </Ind>
      )}
    </div>
  )
}

function tunnelCanStart(tunnel: TunnelStatusView): boolean {
  return tunnel.kind === "start" || (tunnel.kind === "error" && tunnel.retryable_error)
}

function tunnelStatusLabel(kind: TunnelStatusKind): string {
  switch (kind) {
    case "start":
      return "Start"
    case "starting":
      return "Starting"
    case "running":
      return "Online"
    case "error":
      return "Error"
    case "unsupported":
      return "Unsupported"
    case "disabled":
      return "Unavailable"
    default:
      return assertNever(kind)
  }
}

function tunnelTone(kind: TunnelStatusKind): Tone {
  switch (kind) {
    case "running":
      return "ok"
    case "start":
    case "starting":
      return "warn"
    case "error":
      return "bad"
    case "disabled":
    case "unsupported":
      return "off"
    default:
      return assertNever(kind)
  }
}

function tunnelDetail(tunnel: TunnelStatusView): string {
  if (tunnel.kind === "disabled") {
    return "Tunnel unavailable"
  }
  return lineSafeText(tunnel.message ?? tunnel.provider_label ?? tunnel.status_label)
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
