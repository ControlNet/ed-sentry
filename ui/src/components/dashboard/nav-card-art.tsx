import type { CSSProperties } from "react"

export type NavArtKind = "dashboard" | "missions" | "events" | "config" | "about"

// Gauge ticks for the dashboard card: orange on the left, cyan on the right.
const gaugeTicks = [
  "M84.2 37.5L76.2 36.5",
  "M85.4 32.7L78.1 29.7",
  "M87.9 28.4L81.6 23.6",
  "M91.4 24.9L86.6 18.6",
  "M95.7 22.4L92.7 15.1",
  "M100.5 21.2L99.5 13.2",
  "M105.5 21.2L106.5 13.2",
  "M110.3 22.4L113.3 15.1",
  "M114.6 24.9L119.4 18.6",
  "M118.1 28.4L124.4 23.6",
  "M120.6 32.7L127.9 29.7",
  "M121.8 37.5L129.8 36.5",
] as const

const wavePath =
  "M-48 22h10l4 -7l4 14l4 -10l4 6l3 -3h19h10l4 -7l4 14l4 -10l4 6l3 -3h19h10l4 -7l4 14l4 -10l4 6l3 -3h19h10l4 -7l4 14l4 -10l4 6l3 -3h19h10l4 -7l4 14l4 -10l4 6l3 -3h19h10l4 -7l4 14l4 -10l4 6l3 -3h19"

/** Animated line art shown above each navigation card caption. */
export function NavCardArt({ kind }: { readonly kind: NavArtKind }): React.JSX.Element {
  return (
    <span className="art" aria-hidden="true">
      <svg viewBox="0 0 206 44" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
        {renderArt(kind)}
      </svg>
    </span>
  )
}

function renderArt(kind: NavArtKind): React.JSX.Element {
  switch (kind) {
    case "dashboard":
      return (
        <>
          <path className="a-c flow-out" d="M14 40H70M136 40H192" />
          {gaugeTicks.map((d, index) => (
            <path
              key={d}
              className={index < 8 ? "a-o tk" : "a-c tk"}
              style={{ "--i": index } as CSSProperties}
              d={d}
            />
          ))}
          <g className="needle">
            <path className="a-c" d="M103 40L103 25" />
          </g>
          <circle className="a-f pulse" cx="103" cy="40" r="2.6" />
        </>
      )
    case "missions":
      return (
        <>
          <path className="a-c" d="M14 22H72M134 22H192" />
          <g className="ret">
            <circle className="a-c" cx="103" cy="22" r="15" strokeDasharray="9 5" />
          </g>
          <path className="a-c" d="M103 3V11M103 33V41M84 22H92M114 22H122" />
          <path className="a-o br-l" d="M26 15H21V29H26" />
          <path className="a-o br-r" d="M180 15H185V29H180" />
          <circle className="a-f ping" cx="103" cy="22" r="2.6" />
        </>
      )
    case "events":
      return (
        <>
          <g className="wave">
            <path className="a-c" d={wavePath} />
          </g>
          <path className="a-o" d="M14 34H64" />
          <path className="a-o typing" d="M70 34H122" />
          <rect className="a-fo cursor" x="186" y="14" width="6" height="16" />
        </>
      )
    case "config":
      return (
        <>
          <path className="a-c" d="M24 11H182M24 22H182M24 33H182" />
          <rect className="a-fo kn kn1" x="54" y="7" width="8" height="8" />
          <rect className="a-fo kn kn2" x="122" y="18" width="8" height="8" />
          <rect className="a-fo kn kn3" x="82" y="29" width="8" height="8" />
        </>
      )
    case "about":
      return (
        <>
          <path className="a-c" d="M14 22H82M124 22H192" />
          <circle className="a-c ring-p" cx="103" cy="22" r="14" />
          <circle className="a-c" cx="103" cy="22" r="14" />
          <path className="a-o" d="M103 19V30" />
          <circle className="a-f idot" cx="103" cy="14.5" r="1.6" />
          <rect className="a-fo blip bl-l" x="78" y="20.5" width="4" height="3" />
          <rect className="a-fo blip bl-r" x="124" y="20.5" width="4" height="3" />
        </>
      )
    default:
      return assertNever(kind)
  }
}

function assertNever(value: never): never {
  throw new Error(`Unhandled navigation art: ${String(value)}`)
}
