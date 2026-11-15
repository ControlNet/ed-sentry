import clsx from "clsx"
import type { LogLevelKey } from "@/adapters/config"
import { routingGroups } from "@/lib/config-status"
import { ConfigSection } from "./config-form-fields"
import type { ConfigFormState } from "./config-form-model"

type RoutingLevel = 0 | 1 | 2

const routingLevelOptions = [
  { value: 0, label: "Off" },
  { value: 1, label: "Notify" },
  { value: 2, label: "Mention" },
] as const satisfies readonly { readonly value: RoutingLevel; readonly label: string }[]

type ConfigLogLevelsSectionProps = {
  readonly form: ConfigFormState
  readonly onChange: (form: ConfigFormState) => void
}

export function ConfigLogLevelsSection({
  form,
  onChange,
}: ConfigLogLevelsSectionProps): React.JSX.Element {
  const setLogLevel = (key: LogLevelKey, value: number): void => {
    onChange({ ...form, log_levels: { ...form.log_levels, [key]: value } })
  }

  return (
    <ConfigSection id="sys-routing" label="Event routing" title="Event routing">
      <p className="sec2" style={{ margin: "0 0 4px" }}>
        Notify sends a Matrix message. Mention also mentions the Mention user ID.
      </p>
      {routingGroups.map((group) => (
        <div key={group.name}>
          <h3
            style={{
              margin: "20px 0 2px",
              paddingBottom: 6,
              borderBottom: "1px solid rgba(255,255,255,.12)",
              fontSize: 14,
              fontWeight: 500,
              letterSpacing: ".1em",
              textTransform: "uppercase",
              color: "#e8ecf1",
            }}
          >
            {group.name}
          </h3>
          {group.keys.map((key) => (
            <RoutingLevelRow
              key={key}
              label={logLevelLabel(key)}
              value={form.log_levels[key]}
              onChange={(value) => setLogLevel(key, value)}
            />
          ))}
        </div>
      ))}
    </ConfigSection>
  )
}

function RoutingLevelRow({
  label,
  value,
  onChange,
}: {
  readonly label: string
  readonly value: number
  readonly onChange: (value: RoutingLevel) => void
}): React.JSX.Element {
  const activeLevel = displayRoutingLevel(value)
  const name = `${label.toLowerCase().replaceAll(" ", "-")}-routing-level`
  return (
    <div className="frow" style={{ gridTemplateColumns: "minmax(0, 1fr) auto" }}>
      <span style={{ minWidth: 0 }}>{label}</span>
      <div
        role="radiogroup"
        aria-label={`${label} routing level`}
        style={{ display: "flex", gap: 3 }}
      >
        {routingLevelOptions.map((option) => {
          const selected = option.value === activeLevel
          return (
            <label
              key={option.value}
              className={clsx("seg", selected && `seg-on seg-${option.value}`)}
            >
              <input
                aria-label={`${option.label} level ${option.value}`}
                checked={selected}
                className="sr-only"
                name={name}
                type="radio"
                onChange={() => onChange(option.value)}
              />
              {option.label}
            </label>
          )
        })}
      </div>
    </div>
  )
}

function displayRoutingLevel(value: number): RoutingLevel {
  if (value <= 0) {
    return 0
  }
  if (value === 1) {
    return 1
  }
  return 2
}

function logLevelLabel(key: LogLevelKey): string {
  return key
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ")
}
