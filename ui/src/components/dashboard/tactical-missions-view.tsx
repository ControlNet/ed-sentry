import clsx from "clsx"
import { useState } from "react"
import type { AppSnapshot, MissionView } from "@/adapters/dashboard"
import {
  formatCredits,
  formatMinutes,
  parseTimestamp,
  utcDateTime,
  utcHourMinute,
} from "@/lib/format"
import { missionEtaLabel, missionEtaMinutes, sharedKillProgress } from "@/lib/telemetry"
import { type LabelSide, markerLabelLayout, missionTimeline } from "@/lib/timeline"
import { Bar, Ind, Panel } from "./g1-ui"
import {
  destinationLabel,
  missionProgressCurrent,
  missionProgressTotal,
} from "./tactical-mission-utils"

export function TacticalMissionsView({
  snapshot,
  nowMs,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
}): React.JSX.Element {
  const items = snapshot.missions.items
  const [selectedId, setSelectedId] = useState<number | null>(items[0]?.mission_id ?? null)
  const selected = items.find((mission) => mission.mission_id === selectedId) ?? items[0] ?? null

  return (
    <div
      className="g-miss"
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(0, 400px) minmax(0, 1fr)",
        gap: 20,
        alignItems: "start",
      }}
    >
      <div className="col" style={{ gap: 16 }}>
        <Panel label="Active missions" title="Active missions">
          {items.length === 0 ? (
            <p className="sec2" style={{ margin: 0 }}>
              No active missions
            </p>
          ) : (
            <div className="list">
              {items.map((mission) => (
                <MissionListButton
                  key={mission.mission_id}
                  mission={mission}
                  selected={mission.mission_id === selected?.mission_id}
                  onSelect={() => setSelectedId(mission.mission_id)}
                />
              ))}
            </div>
          )}
        </Panel>
        <StackPanel snapshot={snapshot} />
      </div>
      <div className="col" style={{ gap: 16 }}>
        {selected === null ? null : <MissionDetail mission={selected} />}
        <MissionTimelinePanel
          snapshot={snapshot}
          nowMs={nowMs}
          selectedId={selected?.mission_id ?? null}
          onSelect={setSelectedId}
        />
      </div>
    </div>
  )
}

function MissionListButton({
  mission,
  selected,
  onSelect,
}: {
  readonly mission: MissionView
  readonly selected: boolean
  readonly onSelect: () => void
}): React.JSX.Element {
  const total = missionProgressTotal(mission)
  const current = missionProgressCurrent(mission)
  return (
    <button
      type="button"
      className={clsx("mrow", selected && "mrow-on")}
      aria-pressed={selected}
      onClick={onSelect}
    >
      <span style={{ display: "flex", justifyContent: "space-between", gap: 12 }}>
        <span
          style={{
            fontSize: 16,
            fontWeight: 500,
            letterSpacing: ".04em",
            textTransform: "uppercase",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {mission.display_name}
        </span>
        <span style={{ fontWeight: 500, whiteSpace: "nowrap" }}>
          {total > 0 ? `${current}/${total}` : mission.state_label}
        </span>
      </span>
      <span className="dt" style={{ margin: "2px 0 10px" }}>
        {mission.kind_label} · ID {mission.mission_id} · {formatCredits(mission.reward.value)} cr
      </span>
      {total > 0 ? (
        <span className="bar" style={{ display: "block" }}>
          <span
            className="barf"
            style={{ display: "block", width: `${Math.min(100, (current / total) * 100)}%` }}
          />
        </span>
      ) : null}
    </button>
  )
}

function StackPanel({ snapshot }: { readonly snapshot: AppSnapshot }): React.JSX.Element {
  const missions = snapshot.missions
  const progress = sharedKillProgress(missions.items)
  const rate = snapshot.session.kill_total_rate_per_hour.value
  const reward = missions.items.reduce((sum, mission) => sum + mission.reward.value, 0)
  const targets = [
    ...new Set(
      missions.items.flatMap((mission) =>
        mission.target_faction === null || mission.target_faction === undefined
          ? []
          : [mission.target_faction],
      ),
    ),
  ]
  return (
    <Panel label="Stack" title="Stack">
      <div className="trio" style={{ marginTop: 0 }}>
        <div>
          <span className="lab">Completed</span>
          <b>{missions.completed_count}</b>
        </div>
        <div>
          <span className="lab">Active</span>
          <b>{missions.active_count}</b>
        </div>
        <div>
          <span className="lab">Total</span>
          <b>{missions.total_count}</b>
        </div>
      </div>
      {progress.total > 0 ? (
        <div className="row" style={{ marginTop: 6 }}>
          <span className="k">Stack ETA</span>
          <span className="cy" data-testid="mission-stack-eta">
            {missionEtaLabel(progress.remaining, rate)} · {progress.remaining} kills at{" "}
            {rate.toFixed(1)}/h
          </span>
        </div>
      ) : null}
      <div className="row">
        <span className="k">Combined reward</span>
        <span className="or" style={{ fontWeight: 500, whiteSpace: "nowrap" }}>
          {formatCredits(reward)} cr
        </span>
      </div>
      {targets.length === 0 ? null : (
        <div className="row">
          <span className="k">Target faction</span>
          <span className="rd" style={{ textAlign: "right" }}>
            {targets.join(", ")}
          </span>
        </div>
      )}
    </Panel>
  )
}

function MissionDetail({ mission }: { readonly mission: MissionView }): React.JSX.Element {
  const acceptedMs = parseTimestamp(mission.accepted_at)
  const expiryMs = parseTimestamp(mission.expiry)
  const total = missionProgressTotal(mission)
  const current = missionProgressCurrent(mission)
  return (
    <Panel
      label="Mission details"
      title="Mission details"
      right={<Ind tone={mission.state === "active" ? "ok" : "off"}>{mission.state_label}</Ind>}
    >
      <h3
        style={{
          margin: "0 0 12px",
          fontSize: 26,
          fontWeight: 500,
          letterSpacing: ".04em",
          textTransform: "uppercase",
          color: "#e8ecf1",
        }}
      >
        {mission.display_name}
      </h3>
      <div
        className="g-mdet"
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1.2fr) minmax(0, 1fr)",
          gap: 22,
        }}
      >
        <div>
          <DetailRow label="Issuing faction" value={mission.issuing_faction ?? "Unknown"} />
          <DetailRow
            label="Target faction"
            value={mission.target_faction ?? "Unknown"}
            className="rd"
          />
          <DetailRow label="Destination" value={destinationLabel(mission)} className="cy" />
          <DetailRow
            label="Accepted"
            value={acceptedMs === null ? mission.accepted_at_display : utcDateTime(acceptedMs)}
          />
          <DetailRow
            label="Expiry"
            value={
              expiryMs === null ? (mission.expiry_display ?? "Unknown") : utcDateTime(expiryMs)
            }
            className="or"
          />
          <div className="row">
            <span className="k">Reward</span>
            <span style={{ fontSize: 18, fontWeight: 500, color: "#ff8a1a", whiteSpace: "nowrap" }}>
              {formatCredits(mission.reward.value)} cr
            </span>
          </div>
        </div>
        <MissionProgressBox mission={mission} current={current} total={total} />
      </div>
    </Panel>
  )
}

function MissionProgressBox({
  mission,
  current,
  total,
}: {
  readonly mission: MissionView
  readonly current: number
  readonly total: number
}): React.JSX.Element {
  const progress = mission.progress
  if (progress.kind === "none") {
    return (
      <div className="cell" style={{ padding: "14px 18px" }}>
        <span className="lab">Progress</span>
        <p className="sec2" style={{ margin: "8px 0 0" }}>
          No progress tracked
        </p>
      </div>
    )
  }
  const remaining = Math.max(0, total - current)
  return (
    <div className="cell" style={{ padding: "14px 18px" }} data-testid="mission-progress">
      <span className="lab">{progress.kind === "trade" ? "Delivered" : "Kills"}</span>
      <p
        style={{
          margin: "8px 0 0",
          fontSize: 64,
          fontWeight: 500,
          lineHeight: 1,
          color: "#e8ecf1",
        }}
      >
        {current}
        <span className="sec2" style={{ fontSize: 28 }}>
          /{total}
        </span>
      </p>
      <div style={{ margin: "14px 0 10px" }}>
        <Bar percent={total > 0 ? (current / total) * 100 : 0} />
      </div>
      <p style={{ margin: 0 }}>
        <span className="or" style={{ fontWeight: 500 }}>
          {remaining} remaining
        </span>
      </p>
      {progress.kind === "trade" ? (
        <div style={{ marginTop: 8 }}>
          <DetailRow label="Commodity" value={progress.commodity ?? "Unknown"} />
          <DetailRow label="Collected" value={`${progress.collected}/${progress.count}`} />
        </div>
      ) : null}
    </div>
  )
}

function DetailRow({
  label,
  value,
  className,
}: {
  readonly label: string
  readonly value: string
  readonly className?: string
}): React.JSX.Element {
  return (
    <div className="row">
      <span className="k">{label}</span>
      <span
        className={className}
        style={{
          textAlign: "right",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
        title={value}
      >
        {value}
      </span>
    </div>
  )
}

function MissionTimelinePanel({
  snapshot,
  nowMs,
  selectedId,
  onSelect,
}: {
  readonly snapshot: AppSnapshot
  readonly nowMs: number
  readonly selectedId: number | null
  readonly onSelect: (missionId: number) => void
}): React.JSX.Element | null {
  const items = snapshot.missions.items
  const progress = sharedKillProgress(items)
  const etaMinutes =
    progress.total > 0
      ? missionEtaMinutes(progress.remaining, snapshot.session.kill_total_rate_per_hour.value)
      : null
  const etaMs = etaMinutes === null ? null : nowMs + etaMinutes * 60_000
  const timeline = missionTimeline(items, nowMs, etaMs)
  const labels = markerLabelLayout(timeline.nowLeft, timeline.etaLeft)
  if (timeline.rows.length === 0) {
    return null
  }
  const names = new Map(items.map((mission) => [mission.mission_id, mission.display_name]))
  const startDay = new Date(timeline.start).toISOString().slice(0, 10)
  return (
    <Panel
      label="Mission timeline"
      title="Mission timeline"
      right={<span className="lab">{startDay} UTC</span>}
    >
      <div className="tl">
        {timeline.rows.map((row) => {
          const name = names.get(row.missionId) ?? `ID ${row.missionId}`
          const left = formatMinutes(row.remainingSeconds)
          return (
            <button
              key={row.missionId}
              type="button"
              className={clsx("tl-row", row.missionId === selectedId && "tl-sel")}
              aria-pressed={row.missionId === selectedId}
              onClick={() => onSelect(row.missionId)}
            >
              <span className="tl-h">
                <span style={{ minWidth: 0 }}>
                  <span className="tl-name">{name}</span>
                </span>
                <span className="tl-left">
                  <b>{left}</b> <span className="lab">to expiry</span>
                </span>
              </span>
              <span
                className="tl-trk"
                role="img"
                aria-label={`Accepted ${utcHourMinute(row.acceptedMs)}, expires ${utcHourMinute(row.expiryMs)}, ${left} left`}
              >
                <span
                  className="tl-span"
                  style={{ left: `${row.left}%`, width: `${row.width}%` }}
                />
                <span
                  className="tl-done"
                  style={{ left: `${row.left}%`, width: `${Math.max(0, row.elapsedWidth)}%` }}
                />
                <i className="tl-mk tl-now" style={{ left: `${timeline.nowLeft}%` }} />
                {timeline.etaLeft === null ? null : (
                  <i className="tl-mk tl-eta" style={{ left: `${timeline.etaLeft}%` }} />
                )}
              </span>
            </button>
          )
        })}
        <div className="tl-ax">
          <span className="tl-axis">
            {timeline.hours.map((hour) => (
              <span key={hour.label + hour.left} style={{ left: `${hour.left}%` }}>
                {hour.label}
              </span>
            ))}
            <span
              className="mk cy"
              style={{ left: `${timeline.nowLeft}%`, ...markerLabelStyle(labels.nowSide) }}
            >
              Now {utcHourMinute(nowMs)}
            </span>
            {etaMs === null || timeline.etaLeft === null ? null : (
              <span
                className={clsx("mk or", labels.etaLow && "mk-low")}
                style={{ left: `${timeline.etaLeft}%`, ...markerLabelStyle(labels.etaSide) }}
              >
                ETA {utcHourMinute(etaMs)}
              </span>
            )}
          </span>
        </div>
      </div>
      <div className="tl-legend">
        <span>
          <i className="lg" style={{ background: "rgba(60,184,255,.35)" }} />
          Elapsed since accepted
        </span>
        <span>
          <i className="lg" style={{ background: "#1f242d", borderRight: "2px solid #ff8a1a" }} />
          Remaining to expiry
        </span>
        <span>
          <i className="lg" style={{ width: 2, background: "#3cb8ff" }} />
          Now
        </span>
        {etaMinutes === null ? null : (
          <span>
            <i className="lg" style={{ width: 2, background: "#ff8a1a" }} />
            Stack ETA +
            {missionEtaLabel(progress.remaining, snapshot.session.kill_total_rate_per_hour.value)}
          </span>
        )}
      </div>
    </Panel>
  )
}

/** Draws an axis marker label before ("left") or after ("right") its marker. */
function markerLabelStyle(side: LabelSide): React.CSSProperties {
  return side === "left"
    ? { transform: "translateX(-100%)", marginLeft: -6 }
    : { transform: "none", marginLeft: 6 }
}
