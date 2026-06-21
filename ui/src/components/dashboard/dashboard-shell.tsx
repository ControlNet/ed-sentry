import { CircleDot, RefreshCw } from "lucide-react"
import { useState } from "react"
import type { AppSnapshot, DashboardAdapter, DashboardConnectionState } from "@/adapters/dashboard"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { ConfigPanel } from "./config-panel"
import { ConnectionIcon, StatusPill } from "./dashboard-status"
import { EventFeed } from "./event-feed"
import { HealthPanel } from "./health-panel"
import { MetricGrid } from "./metric-grid"
import { MissionPanel } from "./mission-panel"
import { ServiceStatusPanel } from "./service-status-panel"
import { ShellNavigation, type ShellView } from "./shell-navigation"
import { SourcePanel } from "./source-panel"
import { WarningRail } from "./warning-rail"

type DashboardShellProps = {
  readonly snapshot: AppSnapshot
  readonly adapter: DashboardAdapter
  readonly connection: DashboardConnectionState
  readonly isRefreshing: boolean
  readonly onRefresh: () => void
}

export function DashboardShell({
  snapshot,
  adapter,
  connection,
  isRefreshing,
  onRefresh,
}: DashboardShellProps): React.JSX.Element {
  const [activeView, setActiveView] = useState<ShellView>("dashboard")
  const sessionTitle = `${snapshot.session.commander ?? "Unknown commander"} in ${
    snapshot.session.system ?? "Unknown system"
  }`

  return (
    <div className="min-h-[100dvh] bg-background text-foreground">
      <div className="grid min-h-[100dvh] lg:grid-cols-[var(--shell-nav-width)_minmax(0,1fr)]">
        <ShellNavigation
          connection={connection}
          activeView={activeView}
          onViewChange={setActiveView}
        />
        <main className="min-w-0">
          <div className="mx-auto flex w-full max-w-[var(--content-max-width)] flex-col gap-6 px-4 py-4 sm:px-6 lg:px-6">
            <header className="flex min-h-[var(--shell-topbar-height)] flex-col gap-4 border-b pb-4 md:flex-row md:items-center md:justify-between">
              <div className="min-w-0">
                <p className="text-xs font-bold uppercase tracking-[0.06em] text-muted-foreground">
                  ed-sentry dashboard
                </p>
                <h1 className="mt-1 text-2xl font-semibold tracking-normal">AFK monitor</h1>
                <p className="mt-1 truncate text-sm text-muted-foreground">{sessionTitle}</p>
                <div className="mt-2 flex flex-wrap gap-2">
                  <Badge variant="outline">{snapshot.session.ship ?? "Unknown ship"}</Badge>
                  <Badge variant="outline">{snapshot.session.mode ?? "Unknown mode"}</Badge>
                  <Badge variant="outline">{snapshot.session.elapsed_display}</Badge>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <StatusPill kind={snapshot.web.kind} label={snapshot.web.status_label} />
                <Button type="button" variant="outline" onClick={onRefresh} disabled={isRefreshing}>
                  <RefreshCw aria-hidden="true" className={cn(isRefreshing && "animate-spin")} />
                  Refresh
                </Button>
              </div>
            </header>

            <section
              aria-label="Connection state"
              className="grid gap-3 rounded-md border bg-card p-4 md:grid-cols-[minmax(0,1fr)_auto]"
            >
              <div className="flex min-w-0 items-start gap-3">
                <ConnectionIcon status={connection.status} />
                <div className="min-w-0">
                  <h2 className="font-semibold tracking-normal">{connection.label}</h2>
                  <p className="mt-1 text-sm text-muted-foreground">{connection.detail}</p>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2 md:justify-end">
                <Badge variant="outline">{snapshot.generated_at_display}</Badge>
                <Badge variant={snapshot.session.active ? "default" : "secondary"}>
                  <CircleDot aria-hidden="true" />
                  {snapshot.session.status_label}
                </Badge>
              </div>
            </section>

            {activeView === "dashboard" ? (
              <section className="grid items-start gap-4 xl:grid-cols-12">
                <div className="grid gap-4 xl:col-span-8">
                  <MetricGrid session={snapshot.session} />
                  <EventFeed events={snapshot.event_feed} />
                  <MissionPanel
                    missions={snapshot.missions.items}
                    status={snapshot.missions.status_label}
                  />
                </div>
                <div className="grid gap-4 xl:col-span-4">
                  <HealthPanel session={snapshot.session} events={snapshot.event_feed} />
                  <WarningRail snapshot={snapshot} connection={connection} />
                  <SourcePanel snapshot={snapshot} />
                  <ServiceStatusPanel matrix={snapshot.matrix} web={snapshot.web} />
                </div>
              </section>
            ) : (
              <ConfigPanel adapter={adapter} />
            )}
          </div>
        </main>
      </div>
    </div>
  )
}
