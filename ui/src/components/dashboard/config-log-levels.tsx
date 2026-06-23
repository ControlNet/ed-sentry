import { ListFilter } from "lucide-react"
import { type LogLevelKey, logLevelKeys } from "@/adapters/config"
import { cn } from "@/lib/utils"
import type { ConfigFormState } from "./config-form-model"

type RoutingLevel = 0 | 1 | 2

type RoutingLevelOption = {
  readonly value: RoutingLevel
  readonly label: string
  readonly detail: string
  readonly badgeClassName: string
  readonly selectedClassName: string
}

const routingLevelOptions = [
  {
    value: 0,
    label: "Off",
    detail: "Do not route this event.",
    badgeClassName: "border-slate-500/30 bg-slate-500/10 text-slate-300",
    selectedClassName:
      "border-slate-400/50 bg-slate-400/15 text-slate-100 shadow-[0_0_14px_rgb(148_163_184/0.14)]",
  },
  {
    value: 1,
    label: "Notify",
    detail: "Send a standard notification.",
    badgeClassName: "border-cyan-500/30 bg-cyan-500/10 text-cyan-300",
    selectedClassName:
      "border-cyan-400/50 bg-cyan-500/20 text-cyan-100 shadow-[0_0_14px_rgb(34_211_238/0.16)]",
  },
  {
    value: 2,
    label: "Mention",
    detail: "Notify and mention the Matrix user.",
    badgeClassName: "border-orange-500/30 bg-orange-500/10 text-orange-300",
    selectedClassName:
      "border-orange-500/50 bg-orange-500/20 text-orange-100 shadow-[0_0_14px_rgb(249_115_22/0.16)]",
  },
] as const satisfies readonly RoutingLevelOption[]

type ConfigLogLevelsSectionProps = {
  readonly form: ConfigFormState
  readonly onChange: (form: ConfigFormState) => void
}

export function ConfigLogLevelsSection({
  form,
  onChange,
}: ConfigLogLevelsSectionProps): React.JSX.Element {
  const setLogLevel = (key: LogLevelKey, value: number): void => {
    onChange({
      ...form,
      log_levels: {
        ...form.log_levels,
        [key]: value,
      },
    })
  }

  return (
    <section aria-label="Log level settings" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-[#060a11] px-2 text-[10px] font-bold uppercase tracking-widest text-orange-500">
        <ListFilter aria-hidden="true" className="size-3" />
        Event routing levels
      </h3>
      <p className="mt-2 text-xs text-slate-500">
        Choose how each event is routed: off, normal notification, or notification with a Matrix
        mention.
      </p>
      <div className="grid gap-4 sm:grid-cols-2">
        {logLevelKeys.map((key) => (
          <RoutingLevelSlider
            key={key}
            label={logLevelLabel(key)}
            value={form.log_levels[key]}
            onChange={(value) => setLogLevel(key, value)}
          />
        ))}
      </div>
    </section>
  )
}

function RoutingLevelSlider({
  label,
  value,
  onChange,
}: {
  readonly label: string
  readonly value: number
  readonly onChange: (value: RoutingLevel) => void
}): React.JSX.Element {
  const activeLevel = displayRoutingLevel(value)
  const activeOption = routingLevelOption(activeLevel)
  const descriptionId = `${label.toLowerCase().replaceAll(" ", "-")}-routing-description`

  return (
    <div className="border border-slate-800/70 bg-slate-950/40 p-3 transition-colors hover:border-orange-500/40">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <span className="block font-mono text-[10px] uppercase tracking-widest text-slate-400">
            {label}
          </span>
          <span id={descriptionId} className="mt-1 block text-xs text-slate-500">
            {activeOption.detail}
          </span>
        </div>
        <span
          className={cn(
            "shrink-0 border px-2 py-1 font-mono text-[10px] uppercase tracking-widest",
            activeOption.badgeClassName,
          )}
        >
          {activeOption.label}
        </span>
      </div>
      <div
        aria-describedby={descriptionId}
        aria-label={`${label} routing level`}
        className="mt-3 grid grid-cols-3 gap-1 rounded-sm border border-slate-800 bg-black/40 p-1"
        role="radiogroup"
      >
        {routingLevelOptions.map((option) => {
          const selected = option.value === activeLevel

          return (
            <label
              className={cn(
                "cursor-pointer border border-transparent px-2 py-1.5 text-center font-mono text-[10px] uppercase tracking-widest transition-colors has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-orange-500/60",
                selected
                  ? option.selectedClassName
                  : "text-slate-500 hover:border-slate-700 hover:bg-slate-900/80 hover:text-slate-200",
              )}
              key={option.value}
            >
              <input
                aria-label={`${option.label} level ${option.value}`}
                checked={selected}
                className="sr-only"
                name={`${descriptionId}-level`}
                type="radio"
                onChange={() => onChange(option.value)}
              />
              <span className="block text-[11px]">{option.label}</span>
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

function routingLevelOption(level: RoutingLevel): RoutingLevelOption {
  switch (level) {
    case 0:
      return routingLevelOptions[0]
    case 1:
      return routingLevelOptions[1]
    case 2:
      return routingLevelOptions[2]
  }
}

function logLevelLabel(key: LogLevelKey): string {
  return key
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ")
}
