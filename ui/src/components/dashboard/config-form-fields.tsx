import clsx from "clsx"
import { type JSX, type ReactNode, useState } from "react"

type NumberFieldProps = {
  readonly label: string
  readonly value: number
  readonly min?: number | undefined
  readonly max?: number | undefined
  readonly step?: number | undefined
  readonly onChange: (value: number) => void
}

type TextFieldProps = {
  readonly label: string
  readonly value: string
  readonly type?: "text" | "password"
  readonly placeholder?: string
  readonly autoComplete?: string
  readonly onChange: (value: string) => void
}

type SelectOption = {
  readonly value: string
  readonly label: string
}

type SelectFieldProps = {
  readonly label: string
  readonly value: string
  readonly options: readonly SelectOption[]
  readonly onChange: (value: string) => void
}

type ToggleFieldProps = {
  readonly label: string
  readonly checked: boolean
  readonly onChange: (checked: boolean) => void
}

/** One settings section: a panel with an orange-ruled title. */
export function ConfigSection({
  id,
  label,
  title,
  children,
}: {
  readonly id: string
  readonly label: string
  readonly title: string
  readonly children: ReactNode
}): JSX.Element {
  return (
    <section className="pnl sys-sec" id={id} aria-label={label}>
      <h2 className="pt">
        <span>{title}</span>
      </h2>
      {children}
    </section>
  )
}

export function TextField({
  label,
  value,
  type = "text",
  placeholder,
  autoComplete,
  onChange,
}: TextFieldProps): JSX.Element {
  return (
    <label className="frow">
      <span>{label}</span>
      <input
        aria-label={label}
        className="inp"
        type={type}
        value={value}
        placeholder={placeholder}
        autoComplete={autoComplete}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </label>
  )
}

export type NumberFieldState = {
  readonly rawValue: string
  readonly syncedValue: number
}

export function createNumberFieldState(value: number): NumberFieldState {
  return { rawValue: String(value), syncedValue: value }
}

export function parseNumberFieldInput(raw: string): number | null {
  const trimmed = raw.trim()
  if (trimmed.length === 0) {
    return null
  }
  const parsed = Number(trimmed)
  return Number.isFinite(parsed) ? parsed : null
}

export function syncNumberFieldState(state: NumberFieldState, nextValue: number): NumberFieldState {
  if (nextValue === state.syncedValue) {
    return state
  }
  const currentParsed = parseNumberFieldInput(state.rawValue)
  if (currentParsed !== null && currentParsed === nextValue) {
    return { rawValue: state.rawValue, syncedValue: nextValue }
  }
  return { rawValue: String(nextValue), syncedValue: nextValue }
}

export function applyNumberFieldInput(
  state: NumberFieldState,
  nextRaw: string,
): { readonly state: NumberFieldState; readonly parsedValue: number | null } {
  const parsedValue = parseNumberFieldInput(nextRaw)
  return {
    state: {
      rawValue: nextRaw,
      syncedValue: state.syncedValue,
    },
    parsedValue,
  }
}

export function NumberFieldView({
  label,
  rawValue,
  invalid,
  min,
  max,
  step = 1,
  onRawChange,
}: {
  readonly label: string
  readonly rawValue: string
  readonly invalid: boolean
  readonly min?: number | undefined
  readonly max?: number | undefined
  readonly step?: number | undefined
  readonly onRawChange: (nextRaw: string) => void
}): JSX.Element {
  return (
    <label className="frow">
      <span>{label}</span>
      <input
        aria-label={label}
        aria-invalid={invalid || undefined}
        className="inp"
        type="number"
        value={rawValue}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onRawChange(event.currentTarget.value)}
      />
    </label>
  )
}

export function NumberField({
  label,
  value,
  min,
  max,
  step = 1,
  onChange,
}: NumberFieldProps): JSX.Element {
  const [fieldState, setFieldState] = useState(() => createNumberFieldState(value))
  const syncedState = syncNumberFieldState(fieldState, value)
  if (syncedState !== fieldState) {
    setFieldState(syncedState)
  }

  const handleChange = (nextRaw: string): void => {
    const next = applyNumberFieldInput(syncedState, nextRaw)
    setFieldState(next.state)
    if (next.parsedValue !== null) {
      onChange(next.parsedValue)
    }
  }

  return (
    <NumberFieldView
      label={label}
      rawValue={syncedState.rawValue}
      invalid={parseNumberFieldInput(syncedState.rawValue) === null}
      min={min}
      max={max}
      step={step}
      onRawChange={handleChange}
    />
  )
}

export function SelectField({ label, value, options, onChange }: SelectFieldProps): JSX.Element {
  return (
    <label className="frow">
      <span>{label}</span>
      <select
        aria-label={label}
        className="inp"
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  )
}

export function ToggleField({ label, checked, onChange }: ToggleFieldProps): JSX.Element {
  return (
    <div className="frow">
      <span>{label}</span>
      <span>
        <button
          type="button"
          role="switch"
          aria-label={label}
          aria-checked={checked}
          className={clsx("sw", checked && "sw-on")}
          onClick={() => onChange(!checked)}
        />
      </span>
    </div>
  )
}

export function FieldMessage({
  tone,
  children,
}: {
  readonly tone: "info" | "success" | "warning" | "error"
  readonly children: ReactNode
}): JSX.Element {
  return <p className={clsx("note", `note-${tone}`)}>{children}</p>
}
