import QRCode from "qrcode"
import { useEffect, useState } from "react"
import { createPortal } from "react-dom"

type TunnelLinkQrProps = {
  readonly url: string
  readonly anchor: HTMLElement | null
}

const QR_BOX_SIZE = 128
const VIEWPORT_MARGIN = 8
const ANCHOR_GAP = 4

export function TunnelLinkQr({ url, anchor }: TunnelLinkQrProps): React.JSX.Element | null {
  const [dataUrl, setDataUrl] = useState<string | null>(null)
  const [position, setPosition] = useState<React.CSSProperties | null>(null)

  useEffect(() => {
    let active = true
    void QRCode.toDataURL(url, {
      errorCorrectionLevel: "M",
      margin: 1,
      scale: 4,
    }).then((nextDataUrl) => {
      if (active) {
        setDataUrl(nextDataUrl)
      }
    })
    return () => {
      active = false
    }
  }, [url])

  useEffect(() => {
    if (anchor === null) {
      return
    }
    const updatePosition = (): void => {
      const rect = anchor.getBoundingClientRect()
      const maxLeft = Math.max(VIEWPORT_MARGIN, window.innerWidth - QR_BOX_SIZE - VIEWPORT_MARGIN)
      const maxTop = Math.max(VIEWPORT_MARGIN, window.innerHeight - QR_BOX_SIZE - VIEWPORT_MARGIN)
      const preferredTop = rect.bottom + ANCHOR_GAP
      const flippedTop = rect.top - QR_BOX_SIZE - ANCHOR_GAP
      const visibleTop =
        preferredTop + QR_BOX_SIZE + VIEWPORT_MARGIN > window.innerHeight
          ? flippedTop
          : preferredTop
      setPosition({
        left: clamp(rect.left, VIEWPORT_MARGIN, maxLeft),
        top: clamp(visibleTop, VIEWPORT_MARGIN, maxTop),
      })
    }
    updatePosition()
    window.addEventListener("resize", updatePosition)
    window.addEventListener("scroll", updatePosition, true)
    return () => {
      window.removeEventListener("resize", updatePosition)
      window.removeEventListener("scroll", updatePosition, true)
    }
  }, [anchor])

  if (position === null) {
    return null
  }

  return createPortal(
    <div
      aria-label="Tunnel link QR code"
      className="fixed z-[9999] rounded-sm border border-border-strong bg-surface-panel p-2 shadow-[0_0_24px_rgba(0,0,0,0.55)]"
      data-testid="tunnel-link-qr"
      role="img"
      style={position}
    >
      {dataUrl === null ? (
        <div className="size-28 animate-pulse bg-surface-raised" />
      ) : (
        <img
          alt="QR code for tunnel link"
          className="size-28"
          height={112}
          src={dataUrl}
          width={112}
        />
      )}
    </div>,
    document.body,
  )
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max)
}
