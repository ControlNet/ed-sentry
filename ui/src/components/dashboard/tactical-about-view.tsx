import { ExternalLink, Github, Info, ShieldAlert } from "lucide-react"
import { handleExternalLinkClick } from "./external-link"
import { DataRow, TacticalPanel } from "./tactical-ui"

const SOURCE_REPOSITORY_URL = "https://github.com/ControlNet/ed-sentry"
const AUTHOR_INARA_URL = "https://inara.cz/elite/cmdr/78197/"

export function TacticalAboutView(): React.JSX.Element {
  return (
    <div className="mx-auto max-w-2xl pb-12 animate-in fade-in duration-500">
      <header className="relative mb-8 flex flex-col items-center justify-center border-b border-orange-500/20 pb-8 text-center">
        <div className="pointer-events-none absolute inset-0 rounded-full bg-orange-500/5 blur-[80px]" />
        <h2 className="text-shadow-glow mb-2 text-4xl font-black uppercase tracking-[0.3em] text-slate-100">
          ed-sentry
        </h2>
        <p className="max-w-lg font-mono text-xs uppercase tracking-[0.2em] text-orange-500/80">
          Elite Dangerous AFK Sentry
        </p>
      </header>

      <div className="space-y-6">
        <TacticalPanel title="System Information" icon={Info}>
          <div className="space-y-3 font-mono text-xs uppercase">
            <DataRow
              label="Current Version"
              value={import.meta.env.VITE_ED_SENTRY_BUILD_VERSION}
              valueClassName="font-bold text-orange-400"
            />
            <div className="flex items-center justify-between gap-3 border-b border-tactical-accent/10 pb-1 font-mono text-[10px]">
              <span className="text-text-muted">Author</span>
              <a
                href={AUTHOR_INARA_URL}
                target="_blank"
                rel="noopener noreferrer"
                onClick={handleExternalLinkClick}
                className="flex min-w-0 items-center gap-1.5 text-orange-400 transition-colors hover:text-orange-300 focus-visible:outline focus-visible:outline-1 focus-visible:outline-offset-2 focus-visible:outline-orange-400"
              >
                <span className="truncate">CMDR ControlNet</span>
                <ExternalLink aria-hidden="true" className="size-3 shrink-0" />
              </a>
            </div>
            <DataRow label="Software License" value="GNU Affero General Public License v3.0" />
            <div className="flex items-center justify-between gap-3 pt-1.5 font-mono text-[10px]">
              <span className="text-text-muted">Git Repository</span>
              <a
                href={SOURCE_REPOSITORY_URL}
                target="_blank"
                rel="noopener noreferrer"
                onClick={handleExternalLinkClick}
                className="flex min-w-0 items-center gap-1.5 text-orange-400 transition-colors hover:text-orange-300 focus-visible:outline focus-visible:outline-1 focus-visible:outline-offset-2 focus-visible:outline-orange-400"
              >
                <Github aria-hidden="true" className="size-4 shrink-0" />
                <span className="truncate">Source Repository</span>
                <ExternalLink aria-hidden="true" className="size-3 shrink-0" />
              </a>
            </div>
          </div>
        </TacticalPanel>

        <TacticalPanel title="Legal & Compliance" icon={ShieldAlert} className="border-rose-900/30">
          <div className="rounded-sm border border-rose-900/30 bg-rose-950/10 p-4 text-justify font-mono text-[9px] leading-loose text-rose-500/80">
            <strong className="mb-2 block text-[10px] uppercase tracking-widest text-rose-500">
              Warning: Unofficial Software
            </strong>
            <span>
              ed-sentry is a third-party application and is not affiliated with, endorsed, or
              sponsored by Frontier Developments plc. All in-game assets, names, and concepts are
              the property of Frontier Developments. This tool strictly adheres to the read-only
              constraints of the local Journal files and does not inject, modify, or automate any
              client-side game processes. Use at your own risk. Fly Dangerously, Commander.
            </span>
          </div>
        </TacticalPanel>
      </div>
    </div>
  )
}
