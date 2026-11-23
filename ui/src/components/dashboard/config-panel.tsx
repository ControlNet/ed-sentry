import clsx from "clsx"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import type { ConfigApiView, DashboardAdapter } from "@/adapters/dashboard"
import {
  type ConfigSectionId,
  configSectionIds,
  configSectionStatus,
  restartNoticeFromConfigView,
} from "@/lib/config-status"
import { useDashboardStore } from "@/store/dashboard-store"
import {
  JournalConfigSection,
  MatrixConfigSection,
  TunnelConfigSection,
  WebConfigSection,
} from "./config-core-sections"
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
import { Ind } from "./g1-ui"
import { TunnelConfigAuthPrompt } from "./tunnel-config-auth-prompt"

type ConfigPanelProps = {
  readonly adapter: DashboardAdapter
}

export type SaveState = "idle" | "pending" | "saving" | "saved" | "error"

type SaveMode = "auto" | "protected"

export const AUTOSAVE_DELAY_MS = 800

export type ConfigLoadState =
  | { readonly status: "loading" }
  | {
      readonly status: "ready"
      readonly view: ConfigApiView
      readonly form: ConfigFormState
      readonly savedForm: ConfigFormState
    }
  | { readonly status: "error"; readonly message: string }

export type SaveFailureAction =
  | { readonly kind: "auth_required"; readonly message: string }
  | { readonly kind: "error"; readonly message: string }

export function canScheduleAutosave(
  canSave: boolean,
  protectedChange: boolean,
  saveState: SaveState,
): boolean {
  return canSave && !protectedChange && saveState === "pending"
}

export function classifySaveFailure(error: unknown): SaveFailureAction {
  const message = error instanceof Error ? error.message : "Config save failed"
  if (isTunnelConfigAuthRequired(message)) {
    return { kind: "auth_required", message }
  }
  return { kind: "error", message }
}

export function ConfigPanel({ adapter }: ConfigPanelProps): React.JSX.Element {
  const [state, setState] = useState<ConfigLoadState>({ status: "loading" })
  const [saveState, setSaveState] = useState<SaveState>("idle")
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const [restartNotice, setRestartNotice] = useState<string | null>(null)
  const [loadRevision, setLoadRevision] = useState(0)
  const formRevisionRef = useRef(0)
  const saveAttemptRef = useRef(0)
  const latestFormRef = useRef<ConfigFormState | null>(null)
  const pendingFormAfterUnlockRef = useRef<ConfigFormState | null>(null)
  const [activeSection, setActiveSection] = useState<ConfigSectionId>("journal")
  // Scroll position produced by an index click. While the pane stays there, keep the clicked
  // section highlighted even if a later section is also visible (e.g. near the bottom).
  const pinnedScrollTopRef = useRef<number | null>(null)

  const jumpToSection = (id: ConfigSectionId): void => {
    setActiveSection(id)
    pinnedScrollTopRef.current = scrollSectionIntoView(id)
  }

  const ready = state.status === "ready"
  useEffect(() => {
    if (!ready) {
      return
    }
    const scroller = document.getElementById(`sys-${configSectionIds[0]}`)?.closest(".scroll")
    if (scroller === null || scroller === undefined) {
      return
    }
    let frame = 0
    const update = (): void => {
      frame = 0
      const pinned = pinnedScrollTopRef.current
      if (pinned !== null) {
        if (Math.abs(scroller.scrollTop - pinned) <= 2) {
          return
        }
        pinnedScrollTopRef.current = null
      }
      const next = sectionInView(scroller)
      if (next !== null) {
        setActiveSection(next)
      }
    }
    const onScroll = (): void => {
      if (frame === 0) {
        frame = requestAnimationFrame(update)
      }
    }
    update()
    scroller.addEventListener("scroll", onScroll, { passive: true })
    return () => {
      scroller.removeEventListener("scroll", onScroll)
      if (frame !== 0) {
        cancelAnimationFrame(frame)
      }
    }
  }, [ready])

  useEffect(() => {
    let active = true
    setState({ status: "loading" })
    setSaveMessage(null)
    adapter
      .loadConfig()
      .then((view) => {
        if (!active) {
          return
        }
        const savedForm = formFromConfig(view.config)
        const preservedForm = pendingFormAfterUnlockRef.current
        pendingFormAfterUnlockRef.current = null
        const form = preservedForm ?? savedForm
        latestFormRef.current = form
        setState({ status: "ready", view, form, savedForm })
        setRestartNotice(restartNoticeFromConfigView(view))
        if (preservedForm !== null && isConfigFormDirty(preservedForm, savedForm)) {
          formRevisionRef.current += 1
          const hasProtected = hasProtectedSecretChange(preservedForm)
          setSaveState("pending")
          setSaveMessage(
            hasProtected
              ? "Tunnel access unlocked · save protected change"
              : "Tunnel access unlocked · autosave pending",
          )
        } else {
          setSaveState(loadRevision > 0 ? "saved" : "idle")
          setSaveMessage(loadRevision > 0 ? "Tunnel access unlocked" : null)
        }
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
  }, [adapter, loadRevision])

  const validationErrors = useMemo(
    () => (state.status === "ready" ? validateConfigForm(state.form) : []),
    [state],
  )
  const dirty = state.status === "ready" && isConfigFormDirty(state.form, state.savedForm)
  const protectedChange = state.status === "ready" && hasProtectedSecretChange(state.form)
  const canSave =
    state.status === "ready" &&
    dirty &&
    validationErrors.length === 0 &&
    state.view.policy.state_changing_enabled &&
    saveState !== "saving"
  const canAutosave = canScheduleAutosave(canSave, protectedChange, saveState)

  const setForm = (form: ConfigFormState): void => {
    if (state.status !== "ready") {
      return
    }
    latestFormRef.current = form
    formRevisionRef.current += 1
    setSaveState("pending")
    setSaveMessage(hasProtectedSecretChange(form) ? null : "Autosave pending")
    setState((current) => (current.status === "ready" ? { ...current, form } : current))
  }

  const save = useCallback(
    async (form: ConfigFormState, revision: number, mode: SaveMode): Promise<void> => {
      const saveAttempt = saveAttemptRef.current + 1
      saveAttemptRef.current = saveAttempt

      setSaveState("saving")
      setSaveMessage(mode === "protected" ? "Saving protected secret change" : "Saving changes")
      try {
        const nextView = await adapter.saveConfig(updateFromForm(form))
        const nextForm = formFromConfig(nextView.config)
        if (saveAttempt !== saveAttemptRef.current) {
          return
        }
        const nextNotice = restartNoticeFromConfigView(nextView)
        setRestartNotice(nextNotice)
        const changedDuringSave = revision !== formRevisionRef.current
        if (!changedDuringSave) {
          latestFormRef.current = nextForm
        }
        setState((current) => {
          if (current.status !== "ready") {
            return current
          }
          if (changedDuringSave) {
            return { ...current, view: nextView, savedForm: nextForm }
          }
          return { status: "ready", view: nextView, form: nextForm, savedForm: nextForm }
        })
        setSaveState(changedDuringSave ? "pending" : "saved")
        setSaveMessage(
          changedDuringSave
            ? "Autosave pending"
            : nextNotice === null
              ? "All changes saved"
              : "Saved · restart required",
        )
        const storeState = useDashboardStore.getState()
        if (storeState.snapshot !== null && storeState.adapter === adapter) {
          void storeState.refresh()
        }
      } catch (error) {
        if (saveAttempt !== saveAttemptRef.current) {
          return
        }
        const failure = classifySaveFailure(error)
        if (failure.kind === "auth_required") {
          pendingFormAfterUnlockRef.current = latestFormRef.current ?? form
          setSaveState("idle")
          setSaveMessage(null)
          setState({ status: "error", message: failure.message })
          return
        }
        setSaveState("error")
        setSaveMessage(failure.message)
      }
    },
    [adapter],
  )

  useEffect(() => {
    if (state.status !== "ready" || !canAutosave) {
      return undefined
    }

    const form = state.form
    const revision = formRevisionRef.current
    const timer = globalThis.setTimeout(() => {
      void save(form, revision, "auto")
    }, AUTOSAVE_DELAY_MS)

    return () => globalThis.clearTimeout(timer)
  }, [canAutosave, save, state])

  const saveProtectedChange = (): void => {
    if (state.status !== "ready" || !canSave) {
      return
    }
    void save(state.form, formRevisionRef.current, "protected")
  }

  const retrySave = (): void => {
    if (state.status !== "ready" || !canSave) {
      return
    }
    void save(state.form, formRevisionRef.current, protectedChange ? "protected" : "auto")
  }

  return (
    <ConfigPanelView
      adapter={adapter}
      state={state}
      saveState={saveState}
      saveMessage={saveMessage}
      restartNotice={restartNotice}
      validationErrors={validationErrors}
      protectedChange={protectedChange}
      canSave={canSave}
      activeSection={activeSection}
      onJumpToSection={jumpToSection}
      onFormChange={setForm}
      onSaveProtectedChange={saveProtectedChange}
      onRetrySave={retrySave}
      onAuthenticated={() => setLoadRevision((revision) => revision + 1)}
    />
  )
}

type ConfigPanelViewProps = {
  readonly adapter: DashboardAdapter
  readonly state: ConfigLoadState
  readonly saveState: SaveState
  readonly saveMessage: string | null
  readonly restartNotice: string | null
  readonly validationErrors: readonly string[]
  readonly protectedChange: boolean
  readonly canSave: boolean
  readonly activeSection: ConfigSectionId
  readonly onJumpToSection: (id: ConfigSectionId) => void
  readonly onFormChange: (form: ConfigFormState) => void
  readonly onSaveProtectedChange: () => void
  readonly onRetrySave: () => void
  readonly onAuthenticated: () => void
}

export function ConfigPanelView({
  adapter,
  state,
  saveState,
  saveMessage,
  restartNotice,
  validationErrors,
  protectedChange,
  canSave,
  activeSection,
  onJumpToSection,
  onFormChange,
  onSaveProtectedChange,
  onRetrySave,
  onAuthenticated,
}: ConfigPanelViewProps): React.JSX.Element {
  if (state.status === "loading") {
    return (
      <section aria-label="Config editor" className="pnl">
        <p className="sec2" style={{ margin: 0 }}>
          Loading config
        </p>
      </section>
    )
  }

  if (state.status === "error") {
    if (isTunnelConfigAuthRequired(state.message)) {
      return <TunnelConfigAuthPrompt adapter={adapter} onAuthenticated={onAuthenticated} />
    }

    return (
      <section aria-label="Config editor" className="pnl">
        <h2 className="pt">
          <span>Config unavailable</span>
        </h2>
        <FieldMessage tone="error">{state.message}</FieldMessage>
      </section>
    )
  }

  const status = configSectionStatus(state.form)
  const savingEnabled = state.view.policy.state_changing_enabled
  const messages = [
    savingEnabled ? null : (
      <FieldMessage key="policy" tone="warning">
        {state.view.policy.state_changing_reason}
      </FieldMessage>
    ),
    ...validationErrors.map((message) => (
      <FieldMessage key={message} tone="error">
        {message}
      </FieldMessage>
    )),
    protectedChange ? (
      <FieldMessage key="protected" tone="warning">
        Secret changes are protected and will not autosave. Save them explicitly when the
        replacement or clear request is ready.
      </FieldMessage>
    ) : null,
    restartNotice === null ? null : (
      <FieldMessage key="restart" tone="warning">
        {restartNotice}
      </FieldMessage>
    ),
  ].filter((message) => message !== null)

  return (
    <section aria-label="Config editor" className="sys">
      <aside className="pnl sys-ix" aria-label="Configuration index">
        <h2 className="pt">
          <span>Sections</span>
        </h2>
        <div className="list">
          {configSectionIds.map((id) => (
            <button
              key={id}
              type="button"
              className={clsx("idx", activeSection === id && "idx-on")}
              aria-current={activeSection === id ? "true" : undefined}
              onClick={() => onJumpToSection(id)}
            >
              <span className="ixt">
                <span className="ixl">{sectionTitles[id]}</span>
                <span className="ixs">
                  <span className={status[id].on ? "sq cy" : "sqo"} />
                  {status[id].text}
                </span>
              </span>
            </button>
          ))}
        </div>
        {savingEnabled ? (
          <div style={{ marginTop: 14 }} data-testid="config-save-state">
            <SaveIndicator
              protectedChange={protectedChange}
              restartRequired={restartNotice !== null}
              saveState={saveState}
              saveMessage={saveMessage}
            />
            {protectedChange ? (
              <button
                type="button"
                className="btn"
                style={{ marginTop: 10 }}
                disabled={!canSave}
                onClick={onSaveProtectedChange}
              >
                Save protected change
              </button>
            ) : saveState === "error" ? (
              <button
                type="button"
                className="btn"
                style={{ marginTop: 10 }}
                disabled={!canSave}
                onClick={onRetrySave}
              >
                Retry
              </button>
            ) : null}
          </div>
        ) : null}
      </aside>

      <div style={{ minWidth: 0 }}>
        {messages.length === 0 ? null : <div className="sys-msgs">{messages}</div>}
        <JournalConfigSection form={state.form} onChange={onFormChange} />
        <MatrixConfigSection
          form={state.form}
          tokenPresent={state.view.config.matrix?.access_token_present ?? false}
          onChange={onFormChange}
        />
        <TunnelConfigSection
          form={state.form}
          passwordPresent={state.view.config.tunnel.config_password_present}
          onChange={onFormChange}
        />
        <WebConfigSection form={state.form} onChange={onFormChange} />
        <ConfigMonitorSection form={state.form} onChange={onFormChange} />
        <ConfigLogLevelsSection form={state.form} onChange={onFormChange} />
      </div>
    </section>
  )
}

const sectionTitles: Readonly<Record<ConfigSectionId, string>> = {
  journal: "Journal",
  matrix: "Matrix",
  tunnel: "Tunnel",
  web: "Web UI",
  monitor: "Monitor",
  routing: "Event routing",
}

function SaveIndicator({
  protectedChange,
  restartRequired,
  saveState,
  saveMessage,
}: {
  readonly protectedChange: boolean
  readonly restartRequired: boolean
  readonly saveState: SaveState
  readonly saveMessage: string | null
}): React.JSX.Element {
  if (protectedChange && saveState !== "saving" && saveState !== "error") {
    return <Ind tone="warn">Protected secret change pending</Ind>
  }
  switch (saveState) {
    case "idle":
      return <Ind tone="ok">{saveMessage ?? "Autosave enabled"}</Ind>
    case "pending":
    case "saving":
      return <Ind tone="warn">{saveMessage ?? "Autosave pending"}</Ind>
    case "saved":
      return (
        <Ind tone={restartRequired ? "warn" : "ok"}>
          {saveMessage ?? (restartRequired ? "Saved · restart required" : "All changes saved")}
        </Ind>
      )
    case "error":
      return <Ind tone="bad">{saveMessage ?? "Config save failed"}</Ind>
    default:
      return assertNever(saveState)
  }
}

/** A section counts as reached once its heading is within this distance of the pane top. */
const SECTION_ACTIVATION_OFFSET_PX = 24

/**
 * Choose the index entry to highlight from section offsets relative to the top of the
 * scroll pane: the last section whose heading has reached the top, or the last section
 * once the pane is scrolled to the bottom (later sections may never reach the top).
 */
export function pickActiveSection(
  sections: readonly { readonly id: ConfigSectionId; readonly top: number }[],
  atBottom: boolean,
): ConfigSectionId {
  const last = sections.at(-1)
  if (atBottom && last !== undefined) {
    return last.id
  }
  let active = sections[0]?.id ?? "journal"
  for (const section of sections) {
    if (section.top > SECTION_ACTIVATION_OFFSET_PX) {
      break
    }
    active = section.id
  }
  return active
}

function sectionInView(scroller: Element): ConfigSectionId | null {
  const padTop = Number.parseFloat(getComputedStyle(scroller).paddingTop) || 0
  const viewTop = scroller.getBoundingClientRect().top + padTop
  const sections = configSectionIds.flatMap((id) => {
    const element = document.getElementById(`sys-${id}`)
    return element === null ? [] : [{ id, top: element.getBoundingClientRect().top - viewTop }]
  })
  if (sections.length === 0) {
    return null
  }
  const atBottom =
    scroller.scrollTop > 0 &&
    scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2
  return pickActiveSection(sections, atBottom)
}

/**
 * Scroll the content pane (not the whole window) so the section starts at the top.
 * Returns the resulting scroll position, which may be clamped near the bottom.
 */
function scrollSectionIntoView(id: ConfigSectionId): number | null {
  const element = document.getElementById(`sys-${id}`)
  const scroller = element?.closest(".scroll")
  if (element === null || element === undefined || scroller === null || scroller === undefined) {
    return null
  }
  const padTop = Number.parseFloat(getComputedStyle(scroller).paddingTop) || 0
  scroller.scrollTop +=
    element.getBoundingClientRect().top - scroller.getBoundingClientRect().top - padTop
  return scroller.scrollTop
}

function assertNever(value: never): never {
  throw new Error(`Unhandled save state: ${String(value)}`)
}

function hasProtectedSecretChange(form: ConfigFormState): boolean {
  return (
    form.token_replacement_input.trim().length > 0 ||
    form.matrix.clear_access_token ||
    form.tunnel_password_replacement_input.trim().length > 0 ||
    form.tunnel.clear_config_password
  )
}

function isTunnelConfigAuthRequired(message: string): boolean {
  return message === "Tunnel config access requires a valid bearer token"
}
