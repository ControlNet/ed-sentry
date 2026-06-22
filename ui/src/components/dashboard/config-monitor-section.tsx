import { Gauge } from "lucide-react"
import { NumberField, ToggleField } from "./config-form-fields"
import type { ConfigFormState } from "./config-form-model"

type ConfigMonitorSectionProps = {
  readonly form: ConfigFormState
  readonly onChange: (form: ConfigFormState) => void
}

export function ConfigMonitorSection({
  form,
  onChange,
}: ConfigMonitorSectionProps): React.JSX.Element {
  const setMonitor = (monitor: ConfigFormState["monitor"]): void => {
    onChange({ ...form, monitor })
  }

  return (
    <section aria-label="Monitor settings" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-[#060a11] px-2 text-[10px] font-bold uppercase tracking-widest text-orange-500">
        <Gauge aria-hidden="true" className="size-3" />
        Monitor thresholds
      </h3>
      <div className="mt-2 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
        <ToggleField
          label="Live status"
          checked={form.monitor.live_status}
          onChange={(live_status) => setMonitor({ ...form.monitor, live_status })}
        />
        <ToggleField
          label="Dynamic title"
          checked={form.monitor.dynamic_title}
          onChange={(dynamic_title) => setMonitor({ ...form.monitor, dynamic_title })}
        />
        <ToggleField
          label="UTC timestamps"
          checked={form.monitor.use_utc}
          onChange={(use_utc) => setMonitor({ ...form.monitor, use_utc })}
        />
        <NumberField
          label="Warn kill rate"
          value={form.monitor.warn_kill_rate}
          min={0}
          max={1000}
          onChange={(warn_kill_rate) => setMonitor({ ...form.monitor, warn_kill_rate })}
        />
        <NumberField
          label="Duplicate max"
          value={form.monitor.duplicate_max}
          min={0}
          max={1000}
          onChange={(duplicate_max) => setMonitor({ ...form.monitor, duplicate_max })}
        />
        <NumberField
          label="Minimum scan level"
          value={form.monitor.min_scan_level}
          min={0}
          max={3}
          onChange={(min_scan_level) => setMonitor({ ...form.monitor, min_scan_level })}
        />
        <NumberField
          label="Poll interval ms"
          value={form.monitor.poll_interval_ms}
          min={100}
          max={60_000}
          step={100}
          onChange={(poll_interval_ms) => setMonitor({ ...form.monitor, poll_interval_ms })}
        />
        <NumberField
          label="No kills warning minutes"
          value={form.monitor.warn_no_kills_minutes}
          min={0}
          max={1440}
          onChange={(warn_no_kills_minutes) =>
            setMonitor({ ...form.monitor, warn_no_kills_minutes })
          }
        />
        <NumberField
          label="Warning cooldown minutes"
          value={form.monitor.warn_cooldown_minutes}
          min={0}
          max={1440}
          onChange={(warn_cooldown_minutes) =>
            setMonitor({ ...form.monitor, warn_cooldown_minutes })
          }
        />
      </div>
    </section>
  )
}
