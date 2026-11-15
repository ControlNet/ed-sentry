import clsx from "clsx"
import type { CSSProperties } from "react"
import type { AppSnapshot, MissionView, ServiceStatusKind } from "@/adapters/dashboard"
import { eventLevelTone, eventTypeLabel } from "@/lib/events"
import { formatCredits, formatDuration, parseTimestamp, utcClock } from "@/lib/format"
import {
  formatTrend,
  missionEtaLabel,
  noKillsProgress,
  rateDisplay,
  rateScaleMax,
  readinessSummary,
  scalePercent,
  sharedKillProgress,
  trendPercent,
} from "@/lib/telemetry"
import { useDashboardStore } from "@/store/dashboard-store"
import { assertNever, lineSafeText, sourceDetail } from "./dashboard-helpers"
import { handleExternalLinkClick } from "./external-link"
import { Bar, Ind, iconPaths, Panel, Segs, StrokeIcon, serviceTone, type Tone } from "./g1-ui"
import { missionProgressCurrent, missionProgressTotal } from "./tactical-mission-utils"
import { TunnelServiceLine } from "./tunnel-service-line"

type ChecklistRow = AppSnapshot["afk_checklist"]["rows"][number]

const KILL_SCALE_BASE = 40
const SCAN_SCALE_BASE = 10
const RING_SEGMENTS = 36
const RECENT_EVENT_COUNT = 5

export function TacticalTelemetryView({
  snapshot,
  nowMs,
  onOpenEvents,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
  readonly onOpenEvents: () => void
}): React.JSX.Element {
  const killProgress = sharedKillProgress(snapshot.missions.items)
  const eta = missionEtaLabel(
    killProgress.remaining,
    snapshot.session.kill_total_rate_per_hour.value,
  )
  return (
    <div className="dash">
      <div className="col">
        <CombatPanel snapshot={snapshot} />
        <ShipPanel snapshot={snapshot} />
      </div>
      <div className="col">
        <MissionRing completed={killProgress.completed} total={killProgress.total} eta={eta} />
        <SinceLastKillPanel snapshot={snapshot} nowMs={nowMs} />
        <MissionStackPanel snapshot={snapshot} remaining={killProgress.remaining} />
      </div>
      <div className="col">
        <ServicesPanel snapshot={snapshot} />
        <RecentEventsPanel snapshot={snapshot} onOpenEvents={onOpenEvents} />
      </div>
    </div>
  )
}

function CombatPanel({ snapshot }: { readonly snapshot: AppSnapshot }): React.JSX.Element {
  const session = snapshot.session
  const warn = snapshot.thresholds.warn_kill_rate_per_hour
  const lastKillMs = parseTimestamp(session.last_kill_at)
  const lastScanMs = parseTimestamp(session.last_scan_at)
  return (
    <Panel label="Combat" title="Combat">
      <RateBlock
        label="Kills"
        count={session.kills}
        lastMs={lastKillMs}
        recent={session.kill_recent_rate_per_hour.value}
        average={session.kill_total_rate_per_hour.value}
        base={KILL_SCALE_BASE}
        warn={warn > 0 ? warn : null}
        trendUpIsGood
        testId="combat-kills"
      />
      <RateBlock
        label="Scans"
        count={session.scans}
        lastMs={lastScanMs}
        recent={session.scan_recent_rate_per_hour.value}
        average={session.scan_total_rate_per_hour.value}
        base={SCAN_SCALE_BASE}
        warn={null}
        trendUpIsGood={false}
        testId="combat-scans"
      />
      <div style={{ marginTop: 4 }}>
        <div className="kv-row">
          <span className="k">Bounty vouchers</span>
          <span className="kv-big" data-testid="combat-bounty">
            {formatCredits(session.bounty_total.value)}{" "}
            <span className="sec2" style={{ fontSize: 13 }}>
              CR
            </span>
          </span>
        </div>
        <div className="kv-row">
          <span className="k">Faction merits</span>
          <span style={{ whiteSpace: "nowrap" }} data-testid="combat-merits">
            <span className="kv-big">{session.merits}</span>
            {session.merits_to_report > 0 ? (
              <span className="sec2" style={{ fontSize: 13 }}>
                {" "}
                +{session.merits_to_report} to report
              </span>
            ) : null}
          </span>
        </div>
      </div>
    </Panel>
  )
}

function RateBlock({
  label,
  count,
  lastMs,
  recent,
  average,
  base,
  warn,
  trendUpIsGood,
  testId,
}: {
  readonly label: string
  readonly count: number
  readonly lastMs: number | null
  readonly recent: number
  readonly average: number
  readonly base: number
  readonly warn: number | null
  readonly trendUpIsGood: boolean
  readonly testId: string
}): React.JSX.Element {
  const max = rateScaleMax([recent, average, warn ?? 0], base)
  const trend = count === 0 ? null : trendPercent(recent, average)
  const recentText = rateDisplay(count, recent)
  const averageText = rateDisplay(count, average)
  const warnLeft = warn === null ? null : `${scalePercent(warn, max)}%`
  const chipClass = trendUpIsGood && trend !== null && trend >= 0 ? "chip-up" : "chip-dn"
  return (
    <div className="cell kpi" data-testid={testId}>
      <div className="kpi-h">
        <span className="lab">{label}</span>
        {trend === null ? null : (
          <span className={clsx("chip", chipClass)} title="Recent rate vs session average">
            {formatTrend(trend)}
          </span>
        )}
      </div>
      <div className="kpi-b">
        <div>
          <p className="v">{count}</p>
          <p className="u">Last {lastMs === null ? "—" : utcClock(lastMs)}</p>
        </div>
        <div
          className="cmp"
          role="img"
          aria-label={`${label} rate, recent ${recentText} versus session average ${averageText}, scale 0 to ${max} per hour`}
        >
          <span className="lab">Recent</span>
          <span className="trk">
            <b style={{ width: `${scalePercent(recent, max)}%` }} />
            {warnLeft === null ? null : <i className="wt" style={{ left: warnLeft }} />}
          </span>
          <span className="rv or">{recentText}</span>
          <span className="lab">Avg</span>
          <span className="trk">
            <b className="avg" style={{ width: `${scalePercent(average, max)}%` }} />
            {warnLeft === null ? null : <i className="wt" style={{ left: warnLeft }} />}
          </span>
          <span className="rv">{averageText}</span>
          <span />
          <span className="axis">
            <span>0</span>
            {warn === null || warnLeft === null ? (
              <span className="ax-c" style={{ left: "50%" }}>
                {max / 2}
              </span>
            ) : (
              <span className="ax-c rd" style={{ left: warnLeft }}>
                {warn}
              </span>
            )}
            <span className="ax-r" style={{ left: "100%" }}>
              {max}/h
            </span>
          </span>
          <span />
        </div>
      </div>
    </div>
  )
}

function ShipPanel({ snapshot }: { readonly snapshot: AppSnapshot }): React.JSX.Element {
  const session = snapshot.session
  const rows = snapshot.afk_checklist.rows
  const readiness = readinessSummary(rows)
  return (
    <Panel label="Ship status and AFK readiness" title="Ship status" className="grow">
      <div className="shrow">
        <div className="shrow-h">
          <span className="k">Hull</span>
          <span className="or" style={{ fontWeight: 500 }}>
            {session.ship_hull_display}
          </span>
        </div>
        <Segs fraction={session.ship_hull_percent} label={`Hull ${session.ship_hull_display}`} />
      </div>
      <div
        className="shrow"
        style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}
      >
        <span className="k">Shields</span>
        <Ind tone={booleanTone(session.shields_up, "bad")}>{session.shields_display}</Ind>
      </div>
      <div className="shrow">
        <div className="shrow-h">
          <span className="k">Fighter</span>
          <span style={{ display: "inline-flex", alignItems: "center", gap: 14 }}>
            <Ind tone={booleanTone(session.fighter_alive, "bad")}>
              {fighterLabel(session.fighter_alive)}
            </Ind>
            <span className="or" style={{ fontWeight: 500 }}>
              {session.fighter_hull_display}
            </span>
          </span>
        </div>
        <Segs
          fraction={session.fighter_hull_percent}
          label={`Fighter hull ${session.fighter_hull_display}`}
        />
      </div>
      {rows.length === 0 ? null : (
        <>
          <h3 className="subh">
            <span className="k" style={{ color: "#e8ecf1" }}>
              AFK readiness
            </span>
            <span className="lab" data-testid="afk-readiness-count">
              {readiness.pass} / {readiness.total} pass
            </span>
          </h3>
          <ul
            aria-label="AFK readiness checks"
            style={{ margin: 0, padding: 0, listStyle: "none" }}
          >
            {rows.map((row) => (
              <ChecklistItem key={row.id} row={row} />
            ))}
          </ul>
        </>
      )}
    </Panel>
  )
}

function ChecklistItem({ row }: { readonly row: ChecklistRow }): React.JSX.Element {
  return (
    <li className="row" data-testid="afk-checklist-row" data-state={row.state}>
      <span style={{ minWidth: 0 }} title={row.detail}>
        <span className="nm">{row.label}</span>
        <span className="dt">{row.source}</span>
      </span>
      <Ind tone={checklistTone(row.state)}>{checklistStateLabel(row.state)}</Ind>
    </li>
  )
}

function MissionRing({
  completed,
  total,
  eta,
}: {
  readonly completed: number
  readonly total: number
  readonly eta: string
}): React.JSX.Element {
  const lit = total > 0 ? Math.round((completed / total) * RING_SEGMENTS) : 0
  return (
    <section aria-label="Mission kills" className="map" data-testid="telemetry-mission-ring">
      <div className="orb orbit" aria-hidden="true">
        <i />
        <i />
        <i />
      </div>
      <div
        className="orb"
        aria-hidden="true"
        style={{ width: 272, height: 272, border: "1px dashed rgba(60,184,255,.22)" }}
      />
      {Array.from({ length: RING_SEGMENTS }, (_, index) => (
        <span
          // biome-ignore lint/suspicious/noArrayIndexKey: fixed ring positions
          key={index}
          aria-hidden="true"
          className={clsx(
            "seg36",
            index < lit ? "s-on" : "s-off",
            index === lit - 1 && lit < RING_SEGMENTS && "s-head",
          )}
          style={
            {
              "--i": index,
              transform: `rotate(${index * (360 / RING_SEGMENTS)}deg) translateY(-104px)`,
            } as CSSProperties
          }
        />
      ))}
      <div className="core">
        <span className="sweep" aria-hidden="true" />
        <span className="lab">Mission kills</span>
        <b data-testid="telemetry-mission-kill-progress">
          {total > 0 ? (
            <>
              {completed}
              <span>/{total}</span>
            </>
          ) : (
            "—"
          )}
        </b>
        <span className="lab" style={{ marginTop: 4 }} data-testid="telemetry-mission-kill-eta">
          ETA <span className="or">{total > 0 ? eta : "—"}</span>
        </span>
      </div>
    </section>
  )
}

function SinceLastKillPanel({
  snapshot,
  nowMs,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
}): React.JSX.Element {
  const limit = snapshot.thresholds.warn_no_kills_minutes
  const lastKillMs = parseTimestamp(snapshot.session.last_kill_at)
  const progress = lastKillMs === null ? null : noKillsProgress(lastKillMs, nowMs, limit)
  const over = progress?.over ?? false
  const fraction = progress?.fraction ?? 0
  const quarter = (share: number): string => `${Math.round(limit * share * 10) / 10}m`
  return (
    <Panel label="Since last kill" title="Since last kill" live="off">
      <div className="since">
        <b className={clsx(over && "over")} data-testid="since-last-kill">
          {progress === null ? "—" : formatDuration(progress.seconds)}
        </b>
        <span className="lab">
          No-kills warning {limit > 0 ? <span className="rd">{limit}m</span> : "off"}
        </span>
      </div>
      {limit > 0 ? (
        <>
          <div
            className="thr"
            role="img"
            aria-label={`${Math.round(fraction * 100)}% of the no-kills threshold`}
          >
            <b className={clsx(over && "over")} style={{ width: `${fraction * 100}%` }} />
            <i style={{ left: "25%" }} />
            <i style={{ left: "50%" }} />
            <i style={{ left: "75%" }} />
            <i className="lim" style={{ left: "100%" }} />
          </div>
          <span className="axis">
            <span>0</span>
            <span className="ax-c" style={{ left: "25%" }}>
              {quarter(0.25)}
            </span>
            <span className="ax-c" style={{ left: "50%" }}>
              {quarter(0.5)}
            </span>
            <span className="ax-c" style={{ left: "75%" }}>
              {quarter(0.75)}
            </span>
            <span className="ax-r rd" style={{ left: "100%" }}>
              {limit}m
            </span>
          </span>
        </>
      ) : null}
    </Panel>
  )
}

function MissionStackPanel({
  snapshot,
  remaining,
}: {
  readonly snapshot: AppSnapshot
  readonly remaining: number
}): React.JSX.Element {
  const missions = snapshot.missions
  return (
    <Panel
      label="Mission progress"
      title="Mission stack"
      className="grow"
      right={<span className="lab">{remaining} kills left</span>}
    >
      {missions.items.length === 0 ? (
        <p className="sec2" style={{ margin: "4px 0 8px" }}>
          No active missions
        </p>
      ) : (
        missions.items.map((mission) => <MissionLine key={mission.mission_id} mission={mission} />)
      )}
      <div className="trio">
        <div>
          <span className="lab">Completed</span>
          <b>{missions.completed_count}</b>
        </div>
        <div>
          <span className="lab">Active</span>
          <b data-testid="telemetry-active-mission-count">{missions.active_count}</b>
        </div>
        <div>
          <span className="lab">Total</span>
          <b>{missions.total_count}</b>
        </div>
      </div>
    </Panel>
  )
}

function MissionLine({ mission }: { readonly mission: MissionView }): React.JSX.Element {
  const total = missionProgressTotal(mission)
  const current = missionProgressCurrent(mission)
  const issuer = mission.issuing_faction ?? null
  return (
    <div style={{ padding: "4px 0 8px" }} data-testid="telemetry-mission-row">
      <div className="mline">
        <span>
          <span className="nm">{mission.display_name}</span>
          <span className="dt">
            {issuer === null ? "" : `${issuer} · `}ID {mission.mission_id}
          </span>
        </span>
        <span className="or" style={{ fontWeight: 500 }}>
          {total > 0 ? `${current}/${total}` : mission.state_label}
        </span>
      </div>
      {total > 0 ? <Bar percent={(current / total) * 100} /> : null}
    </div>
  )
}

function ServicesPanel({ snapshot }: { readonly snapshot: AppSnapshot }): React.JSX.Element {
  const startTunnel = useDashboardStore((state) => state.startTunnel)
  const journal = snapshot.journal_source
  const journalKind = journalStatusKind(journal.status_label)
  const webHref = webInterfaceHref(snapshot.web)
  return (
    <Panel label="Services" title="Services">
      <ServiceRow
        name="Journal"
        icon={iconPaths.journal}
        detail={sourceDetail(journal.folder, journal.selected_file)}
        status={journal.status_label}
        kind={journalKind}
      />
      <ServiceRow
        name="Matrix"
        icon={iconPaths.matrix}
        detail={lineSafeText(
          snapshot.matrix.room_id ?? snapshot.matrix.message ?? snapshot.matrix.status_label,
        )}
        status={snapshot.matrix.status_label}
        kind={snapshot.matrix.kind}
      />
      <ServiceRow
        name="Web UI"
        icon={iconPaths.web}
        detail={lineSafeText(
          snapshot.web.message ??
            snapshot.web.bind_address ??
            snapshot.web.url ??
            snapshot.web.status_label,
        )}
        detailHref={webHref}
        status={snapshot.web.status_label}
        kind={snapshot.web.kind}
      />
      <TunnelServiceLine tunnel={snapshot.tunnel} onStart={startTunnel} />
    </Panel>
  )
}

function ServiceRow({
  name,
  icon,
  detail,
  detailHref,
  status,
  kind,
}: {
  readonly name: string
  readonly icon: string
  readonly detail: string
  readonly detailHref?: string | null
  readonly status: string
  readonly kind: ServiceStatusKind
}): React.JSX.Element {
  return (
    <div className="row svc" data-service-node={name} data-status-kind={kind}>
      <span style={{ display: "flex", gap: 10, minWidth: 0 }}>
        <StrokeIcon path={icon} className="or" style={{ width: 17, height: 17, marginTop: 3 }} />
        <span style={{ minWidth: 0 }}>
          <span className="nm">{name}</span>
          {detailHref === null || detailHref === undefined ? (
            <span className="dt" title={detail}>
              {detail}
            </span>
          ) : (
            <a
              className="dt"
              style={{ color: "#3cb8ff" }}
              href={detailHref}
              target="_blank"
              rel="noopener noreferrer"
              onClick={handleExternalLinkClick}
            >
              {detail}
            </a>
          )}
        </span>
      </span>
      <Ind tone={serviceTone(kind)} live={kind === "running"}>
        {status}
      </Ind>
    </div>
  )
}

function RecentEventsPanel({
  snapshot,
  onOpenEvents,
}: {
  readonly snapshot: AppSnapshot
  readonly onOpenEvents: () => void
}): React.JSX.Element {
  const events = snapshot.event_feed.slice(0, RECENT_EVENT_COUNT)
  return (
    <Panel
      label="Recent events"
      title="Recent events"
      className="grow"
      right={
        <button type="button" className="link" onClick={onOpenEvents}>
          All events ▶
        </button>
      }
    >
      {events.length === 0 ? (
        <p className="sec2" style={{ margin: 0 }}>
          No dashboard events have arrived.
        </p>
      ) : (
        events.map((event) => {
          const atMs = parseTimestamp(event.timestamp)
          return (
            <div
              key={event.id}
              className="row ev"
              style={{ alignItems: "flex-start", justifyContent: "flex-start" }}
              data-testid="telemetry-event-row"
            >
              <span className="sec2" style={{ width: 66, flexShrink: 0, fontSize: 14 }}>
                {atMs === null ? event.timestamp_display : utcClock(atMs)}
              </span>
              <span style={{ minWidth: 0 }}>
                <span className={`ind t-${eventLevelTone(event.level)}`} style={{ fontSize: 12 }}>
                  {eventTypeLabel(event)} · L{event.level}
                </span>
                <span style={{ display: "block", fontSize: 14, marginTop: 2 }}>
                  {lineSafeText(event.summary)}
                </span>
              </span>
            </div>
          )
        })
      )}
    </Panel>
  )
}

function booleanTone(value: boolean | null | undefined, falseTone: Tone): Tone {
  if (value === true) {
    return "ok"
  }
  if (value === false) {
    return falseTone
  }
  return "off"
}

function fighterLabel(alive: boolean | null | undefined): string {
  if (alive === true) {
    return "Alive"
  }
  if (alive === false) {
    return "Destroyed"
  }
  return "Unknown"
}

function checklistStateLabel(state: ChecklistRow["state"]): string {
  switch (state) {
    case "pass":
      return "Pass"
    case "fail":
      return "Fail"
    case "unknown":
      return "Unknown"
    default:
      return assertNever(state)
  }
}

function checklistTone(state: ChecklistRow["state"]): Tone {
  switch (state) {
    case "pass":
      return "ok"
    case "fail":
      return "bad"
    case "unknown":
      return "off"
    default:
      return assertNever(state)
  }
}

function journalStatusKind(statusLabel: string): ServiceStatusKind {
  const normalized = statusLabel.trim().toLowerCase()
  if (normalized === "watching" || normalized === "running" || normalized === "active") {
    return "running"
  }
  if (normalized === "warning" || normalized === "degraded" || normalized === "starting") {
    return "warning"
  }
  if (normalized === "error" || normalized === "missing") {
    return "error"
  }
  return "disabled"
}

function webInterfaceHref(web: AppSnapshot["web"]): string | null {
  if (web.message !== null || web.url === null || web.url === undefined) {
    return null
  }
  return web.url.replace("://0.0.0.0:", "://localhost:")
}
