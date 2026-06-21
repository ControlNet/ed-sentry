import { AlertTriangle, Loader2, RotateCcw, Save } from "lucide-react"
import { useEffect, useMemo, useState } from "react"
import type { ConfigApiView, DashboardAdapter } from "@/adapters/dashboard"
import { Button } from "@/components/ui/button"
import { JournalConfigSection, MatrixConfigSection, WebConfigSection } from "./config-core-sections"
import { FieldMessage } from "./config-form-fields"
import {
  type ConfigFormState,
  formFromConfig,
  isConfigFormDirty,
  updateFromForm,
  validateConfigForm,
} from "./config-form-model"
import { ConfigLogLevelsSection } from "./config-log-levels"
import { ConfigMonitorSection } from "./config-monitor-section"

type ConfigPanelProps = {
  readonly adapter: DashboardAdapter
}

type ConfigLoadState =
  | { readonly status: "loading" }
  | {
      readonly status: "ready"
      readonly view: ConfigApiView
      readonly form: ConfigFormState
      readonly savedForm: ConfigFormState
    }
  | { readonly status: "error"; readonly message: string }

export function ConfigPanel({ adapter }: ConfigPanelProps): React.JSX.Element {
  const [state, setState] = useState<ConfigLoadState>({ status: "loading" })
  const [saveState, setSaveState] = useState<"idle" | "saving" | "saved" | "error">("idle")
  const [saveMessage, setSaveMessage] = useState<string | null>(null)

  useEffect(() => {
    let active = true
    adapter
      .loadConfig()
      .then((view) => {
        if (!active) {
          return
        }
        const form = formFromConfig(view.config)
        setState({ status: "ready", view, form, savedForm: form })
      })
      .catch((error: unknown) => {
        if (!active) {
          return
        }
        const message = error instanceof Error ? error.message : "Config load failed"
        setState({ status: "error", message })
      })
    return () => {
      active = false
    }
  }, [adapter])

  const validationErrors = useMemo(
    () => (state.status === "ready" ? validateConfigForm(state.form) : []),
    [state],
  )
  const dirty = state.status === "ready" && isConfigFormDirty(state.form, state.savedForm)
  const canSave =
    state.status === "ready" &&
    dirty &&
    validationErrors.length === 0 &&
    state.view.policy.state_changing_enabled &&
    saveState !== "saving"

  const setForm = (form: ConfigFormState): void => {
    if (state.status !== "ready") {
      return
    }
    setSaveState("idle")
    setSaveMessage(null)
    setState({ ...state, form })
  }

  const save = async (): Promise<void> => {
    if (state.status !== "ready" || !canSave) {
      return
    }
    setSaveState("saving")
    setSaveMessage("Saving config")
    try {
      const nextView = await adapter.saveConfig(updateFromForm(state.form))
      const nextForm = formFromConfig(nextView.config)
      setState({ status: "ready", view: nextView, form: nextForm, savedForm: nextForm })
      setSaveState("saved")
      setSaveMessage("Config saved")
    } catch (error) {
      const message = error instanceof Error ? error.message : "Config save failed"
      setSaveState("error")
      setSaveMessage(message)
    }
  }

  if (state.status === "loading") {
    return (
      <section aria-label="Config editor" className="rounded-md border bg-card p-5">
        <div className="flex items-center gap-3 text-sm text-muted-foreground">
          <Loader2 aria-hidden="true" className="size-4 animate-spin text-primary" />
          Loading config
        </div>
      </section>
    )
  }

  if (state.status === "error") {
    return (
      <section aria-label="Config editor" className="grid gap-3 rounded-md border bg-card p-5">
        <div className="flex items-center gap-3">
          <AlertTriangle aria-hidden="true" className="size-5 text-status-danger" />
          <h2 className="text-lg font-semibold tracking-normal">Config unavailable</h2>
        </div>
        <FieldMessage tone="error">{state.message}</FieldMessage>
      </section>
    )
  }

  return (
    <section aria-label="Config editor" className="grid gap-4">
      <div className="flex flex-col gap-3 border-b pb-4 md:flex-row md:items-end md:justify-between">
        <div className="min-w-0">
          <p className="text-xs font-bold uppercase tracking-[0.06em] text-muted-foreground">
            Editable local config
          </p>
          <h1 className="mt-1 text-2xl font-semibold tracking-normal">Config</h1>
          <p className="mt-1 max-w-3xl text-sm text-muted-foreground">
            Changes save to the active local config source. Token fields are write-only.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={!dirty}
            onClick={() => setForm(state.savedForm)}
          >
            <RotateCcw aria-hidden="true" />
            Cancel
          </Button>
          <Button type="button" disabled={!canSave} onClick={() => void save()}>
            {saveState === "saving" ? (
              <Loader2 aria-hidden="true" className="animate-spin" />
            ) : (
              <Save aria-hidden="true" />
            )}
            Save
          </Button>
        </div>
      </div>

      {!state.view.policy.state_changing_enabled ? (
        <FieldMessage tone="warning">{state.view.policy.state_changing_reason}</FieldMessage>
      ) : null}
      {validationErrors.map((message) => (
        <FieldMessage key={message} tone="error">
          {message}
        </FieldMessage>
      ))}
      {saveMessage === null ? null : (
        <FieldMessage
          tone={saveState === "error" ? "error" : saveState === "saved" ? "success" : "info"}
        >
          {saveMessage}
        </FieldMessage>
      )}

      <JournalConfigSection form={state.form} onChange={setForm} />
      <ConfigMonitorSection form={state.form} onChange={setForm} />
      <WebConfigSection form={state.form} onChange={setForm} />
      <MatrixConfigSection
        form={state.form}
        tokenPresent={state.view.config.matrix?.access_token_present ?? false}
        onChange={setForm}
      />
      <ConfigLogLevelsSection form={state.form} onChange={setForm} />
    </section>
  )
}
