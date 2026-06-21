import { ListFilter } from "lucide-react"
import { type LogLevelKey, logLevelKeys } from "@/adapters/config"
import { NumberField, SectionTitle } from "./config-form-fields"
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
    <section aria-label="Log level settings" className="grid gap-4 rounded-md border bg-card p-4">
      <div className="flex items-start gap-3">
        <ListFilter aria-hidden="true" className="mt-1 size-5 text-primary" />
        <SectionTitle
          title="Log levels"
          description="Compact numeric levels for notification and summary event categories."
        />
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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
