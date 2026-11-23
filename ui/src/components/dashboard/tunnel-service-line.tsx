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

export type TunnelStartRequestState =
  | { readonly status: "idle" }
  | { readonly status: "starting" }
  | { readonly status: "error"; readonly message: string }

export async function runTunnelStart(
  onStart: () => Promise<void>,
  onStateChange: (state: TunnelStartRequestState) => void,
): Promise<void> {
  onStateChange({ status: "starting" })
  try {
    await onStart()
    onStateChange({ status: "idle" })
  } catch (error) {
    const message =
      error instanceof Error && error.message.trim().length > 0
        ? error.message
        : "Tunnel start failed"
    onStateChange({ status: "error", message })
  }
}

export function TunnelServiceLine({ tunnel, onStart }: TunnelServiceLineProps): React.JSX.Element {
  const [qrVisible, setQrVisible] = useState(false)
  const [startState, setStartState] = useState<TunnelStartRequestState>({ status: "idle" })
  const tunnelLinkRef = useRef<HTMLAnchorElement | null>(null)
  const localError =
    startState.status === "error" && tunnel.kind !== "running" && tunnel.kind !== "starting"
      ? startState.message
      : null
  const canStart = tunnelCanStart(tunnel) || localError !== null

  const start = async (): Promise<void> => {
    if (!canStart || startState.status === "starting") {
      return
    }
    await runTunnelStart(onStart, setStartState)
  }

  return (
    <TunnelServiceLineView
      tunnel={tunnel}
      startState={startState}
      qrVisible={qrVisible}
      tunnelLinkRef={tunnelLinkRef}
      onQrVisibleChange={setQrVisible}
      onStartClick={() => {
        void start()
      }}
    />
  )
}

type TunnelServiceLineViewProps = {
  readonly tunnel: TunnelStatusView
  readonly startState: TunnelStartRequestState
  readonly qrVisible: boolean
  readonly tunnelLinkRef: React.RefObject<HTMLAnchorElement | null>
  readonly onQrVisibleChange: (visible: boolean) => void
  readonly onStartClick: () => void
}

export function TunnelServiceLineView({
  tunnel,
  startState,
  qrVisible,
  tunnelLinkRef,
  onQrVisibleChange,
  onStartClick,
}: TunnelServiceLineViewProps): React.JSX.Element {
  const localError =
    startState.status === "error" && tunnel.kind !== "running" && tunnel.kind !== "starting"
      ? startState.message
      : null
  const effectiveKind: TunnelStatusKind = localError !== null ? "error" : tunnel.kind
  const canStart = tunnelCanStart(tunnel) || localError !== null
  const publicUrl = tunnel.kind === "running" ? (tunnel.public_url ?? null) : null
  const busy = startState.status === "starting" || tunnel.kind === "starting"
  const detailText = localError !== null ? lineSafeText(localError) : tunnelDetail(tunnel)

  return (
    <div className="row svc" data-service-node="Tunnel" data-status-kind={effectiveKind}>
      <span style={{ display: "flex", gap: 10, minWidth: 0 }}>
        <StrokeIcon
          path={iconPaths.tunnel}
          className="or"
          style={{ width: 17, height: 17, marginTop: 3 }}
        />
        <span style={{ minWidth: 0, position: "relative" }}>
          <span className="nm">Tunnel</span>
          {publicUrl === null ? (
            <span
              className="dt"
              style={effectiveKind === "error" ? { color: "#ffb3ad" } : undefined}
              role={localError !== null ? "alert" : undefined}
              title={detailText}
            >
              {detailText}
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
              onBlur={() => onQrVisibleChange(false)}
              onFocus={() => onQrVisibleChange(true)}
              onPointerEnter={() => onQrVisibleChange(true)}
              onPointerLeave={() => onQrVisibleChange(false)}
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
        <button type="button" className="btn" onClick={onStartClick}>
          {effectiveKind === "error" ? "Retry" : "Start"}
        </button>
      ) : (
        <Ind tone={tunnelTone(effectiveKind)} live={effectiveKind === "running"}>
          {tunnelStatusLabel(effectiveKind)}
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
