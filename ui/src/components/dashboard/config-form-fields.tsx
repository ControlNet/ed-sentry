import type * as React from "react"
import { cn } from "@/lib/utils"

type FieldShellProps = {
  readonly label: string
  readonly description?: string
  readonly children: React.ReactNode
}

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
  readonly disabled?: boolean
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
  readonly description?: string
  readonly onChange: (checked: boolean) => void
}

export function SectionTitle({
  title,
  description,
}: {
  readonly title: string
  readonly description: string
}): React.JSX.Element {
  return (
    <div className="min-w-0">
      <h2 className="font-mono text-[10px] font-bold uppercase tracking-widest text-orange-500">
        {title}
      </h2>
      <p className="mt-1 text-xs text-slate-500">{description}</p>
    </div>
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
    <FieldShell label={label}>
      <input
        aria-label={label}
        className="tactical-input"
        type={type}
        value={value}
        placeholder={placeholder}
        autoComplete={autoComplete}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </FieldShell>
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
    <FieldShell label={label}>
      <input
        aria-label={label}
        className="tactical-input"
        type="number"
        value={String(value)}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </FieldShell>
  )
}

export function SelectField({
  label,
  value,
  options,
  onChange,
}: SelectFieldProps): React.JSX.Element {
  return (
    <FieldShell label={label}>
      <select
        aria-label={label}
        className="tactical-input"
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      >
        {options.map((option) => (
          <option disabled={option.disabled} key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </FieldShell>
  )
}

export function ToggleField({
  label,
  checked,
  description,
  onChange,
}: ToggleFieldProps): React.JSX.Element {
  return (
    <label className="flex min-h-11 items-start gap-3 border border-slate-800/50 bg-slate-900/50 p-2 transition-colors hover:border-orange-500/50">
      <input
        className="mt-1 size-4 accent-orange-500"
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
      <span className="min-w-0">
        <span className="block font-mono text-xs uppercase tracking-widest text-slate-200">
          {label}
        </span>
        {description === undefined ? null : (
          <span className="mt-1 block text-xs text-slate-500">{description}</span>
        )}
      </span>
    </label>
  )
}

export function FieldMessage({
  tone,
  children,
}: {
  readonly tone: "info" | "success" | "warning" | "error"
  readonly children: React.ReactNode
}): React.JSX.Element {
  return (
    <p
      className={cn(
        "border bg-slate-900/50 p-3 text-sm",
        tone === "success" && "border-emerald-800 text-emerald-400",
        tone === "warning" && "border-amber-800 text-amber-400",
        tone === "error" && "border-rose-800 text-rose-400",
        tone === "info" && "border-slate-800 text-slate-400",
      )}
    >
      {children}
    </p>
  )
}

function FieldShell({ label, description, children }: FieldShellProps): React.JSX.Element {
  return (
    <div className="text-sm">
      <span className="mb-2 block font-mono text-[10px] uppercase text-slate-500">{label}</span>
      {children}
      {description === undefined ? null : (
        <span className="mt-1 block text-xs text-slate-500">{description}</span>
      )}
    </div>
  )
}
