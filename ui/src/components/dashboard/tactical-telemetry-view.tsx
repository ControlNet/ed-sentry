import { Activity, ClipboardCheck, Crosshair, Database, Server, Wifi } from "lucide-react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { useDashboardStore } from "@/store/dashboard-store"
import { assertNever } from "./dashboard-helpers"
import { TacticalMissionSummary, TacticalRecentAlerts } from "./tactical-telemetry-summary"
import { JournalServiceLine, MetricValue, ServiceLine } from "./tactical-telemetry-widgets"
import { DataRow, TacticalBadge, type TacticalBadgeTone, TacticalPanel } from "./tactical-ui"
import { TunnelServiceLine } from "./tunnel-service-line"

type ChecklistRow = AppSnapshot["afk_checklist"]["rows"][number]

export function TacticalTelemetryView({
  snapshot,
}: {
  readonly snapshot: AppSnapshot
}): React.JSX.Element {
  const startTunnel = useDashboardStore((state) => state.startTunnel)

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

      <TacticalPanel title="Checklist" icon={ClipboardCheck}>
        <ul aria-label="AFK readiness checks" className="space-y-2">
          {snapshot.afk_checklist.rows.map((row) => (
            <ChecklistItem key={row.id} row={row} />
          ))}
        </ul>
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
            detail={
              snapshot.matrix.room_id ?? snapshot.matrix.message ?? snapshot.matrix.status_label
            }
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
            detailHref={webInterfaceHref(snapshot.web)}
            badge={snapshot.web.status_label}
            statusKind={snapshot.web.kind}
          />
          <TunnelServiceLine tunnel={snapshot.tunnel} onStart={startTunnel} />
        </div>
      </TacticalPanel>

      <TacticalMissionSummary snapshot={snapshot} />
      <TacticalRecentAlerts snapshot={snapshot} />
    </div>
  )
}

function ChecklistItem({ row }: { readonly row: ChecklistRow }): React.JSX.Element {
  return (
    <li
      className={`grid grid-cols-[minmax(0,1fr)_auto] gap-3 border p-2 font-mono ${checklistRowClass(
        row.state,
      )}`}
      data-testid="afk-checklist-row"
    >
      <p className="min-w-0 truncate text-[11px] font-bold uppercase tracking-wider text-text-primary">
        {row.label}
      </p>
      <TacticalBadge tone={checklistBadgeTone(row.state)}>
        {checklistStateLabel(row.state)}
      </TacticalBadge>
    </li>
  )
}

function checklistStateLabel(state: ChecklistRow["state"]): string {
  switch (state) {
    case "pass":
      return "PASS"
    case "fail":
      return "FAIL"
    case "unknown":
      return "UNKNOWN"
    default:
      return assertNever(state)
  }
}

function checklistBadgeTone(state: ChecklistRow["state"]): TacticalBadgeTone {
  switch (state) {
    case "pass":
      return "success"
    case "fail":
      return "danger"
    case "unknown":
      return "default"
    default:
      return assertNever(state)
  }
}

function checklistRowClass(state: ChecklistRow["state"]): string {
  switch (state) {
    case "pass":
      return "border-status-online/25 bg-status-online/5"
    case "fail":
      return "border-status-danger/25 bg-status-danger/5"
    case "unknown":
      return "border-status-neutral/25 bg-surface-raised/30"
    default:
      return assertNever(state)
  }
}

function webInterfaceHref(web: AppSnapshot["web"]): string | null {
  if (web.message !== null || web.url === null || web.url === undefined) {
    return null
  }
  return web.url.replace("://0.0.0.0:", "://localhost:")
}
