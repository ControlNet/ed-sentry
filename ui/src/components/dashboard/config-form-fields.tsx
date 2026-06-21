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
    <div className="border-b pb-3">
      <h2 className="text-lg font-semibold tracking-normal">{title}</h2>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
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
        className={inputClassName}
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
        className={inputClassName}
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

export function ToggleField({
  label,
  checked,
  description,
  onChange,
}: ToggleFieldProps): React.JSX.Element {
  return (
    <label className="flex min-h-11 items-start gap-3 rounded-md border bg-background p-3">
      <input
        className="mt-1 size-4 accent-primary"
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
      <span className="min-w-0">
        <span className="block text-sm font-medium">{label}</span>
        {description === undefined ? null : (
          <span className="mt-1 block text-xs text-muted-foreground">{description}</span>
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
        "rounded-md border bg-background p-3 text-sm",
        tone === "success" && "border-status-online text-status-online",
        tone === "warning" && "border-status-warning text-status-warning",
        tone === "error" && "border-status-danger text-status-danger",
        tone === "info" && "text-muted-foreground",
      )}
    >
      {children}
    </p>
  )
}

function FieldShell({ label, description, children }: FieldShellProps): React.JSX.Element {
  return (
    <div className="grid gap-2 text-sm">
      <span className="font-medium">{label}</span>
      {children}
      {description === undefined ? null : (
        <span className="text-xs text-muted-foreground">{description}</span>
      )}
    </div>
  )
}

const inputClassName =
  "h-9 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-sm text-foreground outline-none transition-colors placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
