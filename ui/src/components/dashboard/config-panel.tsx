import { AlertTriangle, Loader2, Save } from "lucide-react"
import { useEffect, useMemo, useState } from "react"
import type { ConfigApiView, DashboardAdapter } from "@/adapters/dashboard"
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
      <section aria-label="Config editor" className="tactical-config-section">
        <div className="flex items-center gap-3 text-sm text-text-muted">
          <Loader2 aria-hidden="true" className="size-4 animate-spin text-tactical" />
          Loading config
        </div>
      </section>
    )
  }

  if (state.status === "error") {
    return (
      <section aria-label="Config editor" className="tactical-config-section">
        <div className="flex items-center gap-3">
          <AlertTriangle aria-hidden="true" className="size-5 text-status-danger" />
          <h2 className="tactical-overline text-status-danger">Config unavailable</h2>
        </div>
        <FieldMessage tone="error">{state.message}</FieldMessage>
      </section>
    )
  }

  return (
    <section aria-label="Config editor" className="space-y-6 p-2">
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
      <MatrixConfigSection
        form={state.form}
        tokenPresent={state.view.config.matrix?.access_token_present ?? false}
        onChange={setForm}
      />
      <WebConfigSection form={state.form} onChange={setForm} />
      <ConfigMonitorSection form={state.form} onChange={setForm} />
      <ConfigLogLevelsSection form={state.form} onChange={setForm} />
      <div className="mt-6 flex justify-end border-t border-orange-500/10 px-2 pt-4">
        <button
          type="button"
          className="flex items-center gap-2 rounded-sm bg-orange-600 px-5 py-2 text-[10px] font-bold uppercase tracking-widest text-slate-100 shadow-[0_0_10px_rgba(234,88,12,0.4)] transition-all hover:bg-orange-500 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={!canSave}
          onClick={() => void save()}
        >
          {saveState === "saving" ? (
            <Loader2 aria-hidden="true" className="size-3.5 animate-spin" />
          ) : (
            <Save aria-hidden="true" className="size-3.5" />
          )}
          Commit Modifications
        </button>
      </div>
    </section>
  )
}
