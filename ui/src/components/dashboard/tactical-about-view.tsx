import { handleExternalLinkClick } from "./external-link"
import { Panel } from "./g1-ui"

const SOURCE_REPOSITORY_URL = "https://github.com/ControlNet/ed-sentry"
const AUTHOR_INARA_URL = "https://inara.cz/elite/cmdr/78197/"

const dataSources = [
  { name: "Journal (*.log)", detail: "Kills, scans, missions" },
  { name: "Status.json", detail: "Flags and pips for AFK readiness" },
  { name: "Cargo.json", detail: "Ship cargo for AFK readiness" },
] as const

export function TacticalAboutView(): React.JSX.Element {
  return (
    <div
      className="g-about"
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(0, 330px) minmax(0, 1fr)",
        gap: 20,
        alignItems: "start",
        maxWidth: 1180,
      }}
    >
      <div className="col" style={{ gap: 16 }}>
        <aside
          className="pnl"
          aria-label="App emblem"
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            padding: "24px 16px",
          }}
        >
          <div style={{ position: "relative", width: 190, height: 190 }}>
            <div
              className="orb"
              style={{ width: 186, height: 186, border: "1px dashed rgba(60,184,255,.35)" }}
            />
            <div
              className="orb"
              style={{ width: 144, height: 144, border: "1px solid rgba(255,122,0,.3)" }}
            />
            <img
              src="/logo.png"
              alt=""
              width="92"
              height="92"
              style={{ position: "absolute", left: 49, top: 49, objectFit: "contain" }}
            />
          </div>
        </aside>
        <Panel label="Data sources" title="Data sources" className="grow">
          {dataSources.map((source) => (
            <div key={source.name} className="src">
              <div className="src-h">
                <span className="nm">{source.name}</span>
              </div>
              <span className="dt">{source.detail}</span>
            </div>
          ))}
        </Panel>
      </div>
      <article className="pnl" aria-label="System information">
        <p
          style={{
            margin: "6px 0 2px",
            fontSize: 32,
            fontWeight: 500,
            letterSpacing: ".06em",
            textTransform: "uppercase",
            color: "#e8ecf1",
          }}
        >
          ED Sentry
        </p>
        <p
          className="cy"
          style={{
            margin: "0 0 18px",
            fontSize: 16,
            letterSpacing: ".1em",
            textTransform: "uppercase",
          }}
        >
          Elite Dangerous AFK Sentry
        </p>
        <div className="row">
          <span className="k">Current version</span>
          <span data-testid="about-version">{import.meta.env.VITE_ED_SENTRY_BUILD_VERSION}</span>
        </div>
        <div className="row">
          <span className="k">Author</span>
          <a
            href={AUTHOR_INARA_URL}
            target="_blank"
            rel="noopener noreferrer"
            onClick={handleExternalLinkClick}
          >
            CMDR ControlNet
          </a>
        </div>
        <div className="row">
          <span className="k">Software license</span>
          <span style={{ textAlign: "right" }}>GNU Affero General Public License v3.0</span>
        </div>
        <div className="row">
          <span className="k">Source repository</span>
          <a
            href={SOURCE_REPOSITORY_URL}
            target="_blank"
            rel="noopener noreferrer"
            onClick={handleExternalLinkClick}
          >
            github.com/ControlNet/ed-sentry
          </a>
        </div>
        <section
          aria-label="Legal and compliance"
          style={{
            marginTop: 22,
            padding: "14px 16px",
            borderLeft: "3px solid #ff4d40",
            background: "#1a0f10",
          }}
        >
          <h3 className="ind t-bad" style={{ margin: "0 0 8px", fontSize: 14 }}>
            <span className="sq" />
            Unofficial software
          </h3>
          <p style={{ margin: 0, lineHeight: 1.7, color: "#c9ced6" }}>
            ED Sentry is a third-party application. It is not affiliated with, endorsed or sponsored
            by Frontier Developments plc, which owns Elite Dangerous and its names and assets. ED
            Sentry only reads the local files listed under Data sources; it does not inject into the
            game, press keys, relog or automate the client. Use at your own risk.
          </p>
        </section>
      </article>
    </div>
  )
}
