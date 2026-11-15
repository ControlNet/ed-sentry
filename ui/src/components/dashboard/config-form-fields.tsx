import clsx from "clsx"
import type * as React from "react"

type NumberFieldProps = {
  readonly label: string
  readonly value: number
  readonly min?: number
  readonly max?: number
  readonly step?: number
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
  readonly children: React.ReactNode
}): React.JSX.Element {
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
}: TextFieldProps): React.JSX.Element {
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

export function NumberField({
  label,
  value,
  min,
  max,
  step = 1,
  onChange,
}: NumberFieldProps): React.JSX.Element {
  return (
    <label className="frow">
      <span>{label}</span>
      <input
        aria-label={label}
        className="inp"
        type="number"
        value={String(value)}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </label>
  )
}

export function SelectField({
  label,
  value,
  options,
  onChange,
}: SelectFieldProps): React.JSX.Element {
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

export function ToggleField({ label, checked, onChange }: ToggleFieldProps): React.JSX.Element {
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
  readonly children: React.ReactNode
}): React.JSX.Element {
  return <p className={clsx("note", `note-${tone}`)}>{children}</p>
}
