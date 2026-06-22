import { Activity, Crosshair, Database, Server, Shield, Wifi } from "lucide-react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { TacticalMissionSummary, TacticalRecentAlerts } from "./tactical-telemetry-summary"
import { JournalServiceLine, Meter, MetricValue, ServiceLine } from "./tactical-telemetry-widgets"
import { DataRow, TacticalBadge, TacticalPanel } from "./tactical-ui"

export function TacticalTelemetryView({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  return (
    <div className="grid grid-cols-1 gap-4 pb-8 animate-in fade-in duration-500 md:grid-cols-2 lg:grid-cols-4">
      <TacticalPanel
        title="Telemetry Link"
        icon={Wifi}
        rightElement={
          <TacticalBadge tone={snapshot.session.active ? "success" : "warning"}>
            {snapshot.session.status_label}
          </TacticalBadge>
        }
      >
        <div className="space-y-3">
          <DataRow
            label="CMDR"
            value={snapshot.session.commander ?? "Unknown"}
            valueClassName="text-tactical"
          />
          <DataRow label="SHIP" value={snapshot.session.ship ?? "Unknown"} />
          <DataRow label="SYSTEM" value={snapshot.session.system ?? "Unknown"} />
          <DataRow label="MODE" value={snapshot.session.mode ?? "Unknown"} />
          <DataRow
            label="UPTIME"
            value={snapshot.session.elapsed_display}
            valueClassName="text-status-online"
          />
        </div>
      </TacticalPanel>

      <TacticalPanel title="Ship Integrity" icon={Shield}>
        <div className="space-y-5">
          <Meter
            label="Hull plating"
            value={snapshot.session.ship_hull_display}
            percent={snapshot.session.ship_hull_percent}
          />
          <div className="rounded-sm border border-slate-800/50 bg-slate-900/30 p-2">
            <DataRow
              label="Deflector shield"
              value={snapshot.session.shields_display}
              valueClassName={
                snapshot.session.shields_up === false ? "text-status-danger" : "text-tactical"
              }
            />
          </div>
          <Meter
            label="SLF fighter"
            value={snapshot.session.fighter_hull_display}
            percent={snapshot.session.fighter_hull_percent}
            tone="scan"
          />
        </div>
      </TacticalPanel>

      <TacticalPanel title="Combat Analytics" icon={Crosshair}>
        <div className="grid grid-cols-2 gap-3">
          <MetricValue label="Total kills" value={String(snapshot.session.kills)} />
          <MetricValue label="Total scans" value={String(snapshot.session.scans)} />
        </div>
        <div className="mt-4 space-y-2">
          <DataRow
            label="BOUNTY VOUCHERS"
            value={snapshot.session.bounty_total.display}
            valueClassName="text-tactical"
          />
          <DataRow
            label="FACTION MERITS"
            value={`${snapshot.session.merits} pts`}
            valueClassName="text-data-scan"
          />
          <DataRow
            label="KILL RATE / HR"
            value={snapshot.session.kill_recent_rate_per_hour.display}
          />
          <DataRow
            label="SCAN RATE / HR"
            value={snapshot.session.scan_recent_rate_per_hour.display}
          />
        </div>
      </TacticalPanel>

      <TacticalPanel title="Service Nodes" icon={Server}>
        <div className="space-y-4">
          <JournalServiceLine snapshot={snapshot} icon={Database} />
          <ServiceLine
            icon={Activity}
            label="Matrix Relay"
            detail={snapshot.matrix.message ?? snapshot.matrix.status_label}
            badge={snapshot.matrix.status_label}
            statusKind={snapshot.matrix.kind}
          />
          <ServiceLine
            icon={Wifi}
            label="Web Interface"
            detail={
              snapshot.web.message ??
              snapshot.web.bind_address ??
              snapshot.web.url ??
              snapshot.web.status_label
            }
            badge={snapshot.web.status_label}
            statusKind={snapshot.web.kind}
          />
        </div>
      </TacticalPanel>

      <TacticalMissionSummary snapshot={snapshot} />
      <TacticalRecentAlerts snapshot={snapshot} />
    </div>
  )
}
