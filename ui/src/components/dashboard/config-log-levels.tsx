import { ListFilter } from "lucide-react"
import { type LogLevelKey, logLevelKeys } from "@/adapters/config"
import { NumberField } from "./config-form-fields"
import type { ConfigFormState } from "./config-form-model"

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
      <div className="mt-2 grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {logLevelKeys.map((key) => (
          <NumberField
            key={key}
            label={logLevelLabel(key)}
            value={form.log_levels[key]}
            min={0}
            max={5}
            onChange={(value) => setLogLevel(key, value)}
          />
        ))}
      </div>
    </section>
  )
}

function logLevelLabel(key: LogLevelKey): string {
  return key
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ")
}
