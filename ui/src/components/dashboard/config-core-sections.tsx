import { Activity, Database, Server } from "lucide-react"
import { FieldMessage, NumberField, TextField, ToggleField } from "./config-form-fields"
import type { ConfigFormState } from "./config-form-model"
import { isLoopbackHost } from "./config-form-model"

type ConfigSectionProps = {
  readonly form: ConfigFormState
  readonly onChange: (form: ConfigFormState) => void
}

type MatrixSectionProps = ConfigSectionProps & {
  readonly tokenPresent: boolean
}

export function JournalConfigSection({ form, onChange }: ConfigSectionProps): React.JSX.Element {
  return (
    <section aria-label="Journal settings" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-[#060a11] px-2 text-[10px] font-bold uppercase tracking-widest text-orange-500">
        <Database aria-hidden="true" className="size-3" />
        Local ingestion
      </h3>
      <div className="mt-2 grid gap-3 md:grid-cols-[minmax(0,1fr)_12rem]">
        <TextField
          label="Journal folder"
          value={form.journal.folder}
          placeholder="Default journal folder"
          onChange={(folder) => onChange({ ...form, journal: { ...form.journal, folder } })}
        />
        <NumberField
          label="Recent files"
          value={form.journal.recent_files}
          min={1}
          max={100}
          onChange={(recent_files) =>
            onChange({ ...form, journal: { ...form.journal, recent_files } })
          }
        />
      </div>
    </section>
  )
}

export function WebConfigSection({ form, onChange }: ConfigSectionProps): React.JSX.Element {
  return (
    <section aria-label="Web settings" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-[#060a11] px-2 text-[10px] font-bold uppercase tracking-widest text-orange-500">
        <Server aria-hidden="true" className="size-3" />
        Local API gateway
      </h3>
      <div className="mt-2 grid gap-5 md:grid-cols-2 xl:grid-cols-4">
        <ToggleField
          label="Enabled"
          checked={form.web.enabled}
          onChange={(enabled) => onChange({ ...form, web: { ...form.web, enabled } })}
        />
        <ToggleField
          label="Open browser"
          checked={form.web.open_browser}
          onChange={(open_browser) => onChange({ ...form, web: { ...form.web, open_browser } })}
        />
        <TextField
          label="Host"
          value={form.web.host}
          onChange={(host) => onChange({ ...form, web: { ...form.web, host } })}
        />
        <NumberField
          label="Port"
          value={form.web.port}
          min={1}
          max={65_535}
          onChange={(port) => onChange({ ...form, web: { ...form.web, port } })}
        />
      </div>
      {isLoopbackHost(form.web.host) ? null : (
        <FieldMessage tone="warning">
          Non-loopback hosts disable remote state-changing config writes by policy.
        </FieldMessage>
      )}
    </section>
  )
}

export function MatrixConfigSection({
  form,
  tokenPresent,
  onChange,
}: MatrixSectionProps): React.JSX.Element {
  return (
    <section aria-label="Matrix settings" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-[#060a11] px-2 text-[10px] font-bold uppercase tracking-widest text-orange-500">
        <Activity aria-hidden="true" className="size-3" />
        Matrix relay protocol
      </h3>
      <div className="mt-2 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
        <div className="md:col-span-2 xl:col-span-3">
          <ToggleField
            label="Enable Matrix Broadcasting"
            checked={form.matrix.enabled}
            onChange={(enabled) => onChange({ ...form, matrix: { ...form.matrix, enabled } })}
          />
        </div>
        <TextField
          label="Homeserver URI"
          value={form.matrix.homeserver ?? ""}
          onChange={(homeserver) => onChange({ ...form, matrix: { ...form.matrix, homeserver } })}
        />
        <TextField
          label="User ID"
          value={form.matrix.user_id ?? ""}
          onChange={(user_id) => onChange({ ...form, matrix: { ...form.matrix, user_id } })}
        />
        <TextField
          label="Room ID"
          value={form.matrix.room_id ?? ""}
          onChange={(room_id) => onChange({ ...form, matrix: { ...form.matrix, room_id } })}
        />
        <TextField
          label="Mention user ID"
          value={form.matrix.mention_user_id ?? ""}
          onChange={(mention_user_id) =>
            onChange({ ...form, matrix: { ...form.matrix, mention_user_id } })
          }
        />
        <NumberField
          label="Status cadence"
          value={form.matrix.status_update_interval_seconds}
          min={10}
          max={86_400}
          onChange={(status_update_interval_seconds) =>
            onChange({ ...form, matrix: { ...form.matrix, status_update_interval_seconds } })
          }
        />
        <div className="border-t border-slate-800/50 pt-4 md:col-span-2 xl:col-span-3">
          <div className="mb-2 flex justify-between font-mono text-[10px] uppercase">
            <span className="text-slate-500">
              Access Token <span className="ml-1 text-rose-500/80">(WRITE-ONLY)</span>
            </span>
            <span className="font-bold text-emerald-500">
              {tokenPresent ? "TOKEN PRESENT IN VAULT" : "NO TOKEN IN VAULT"}
            </span>
          </div>
          <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_14rem]">
            <TextField
              label="Replace access token"
              type="password"
              value={form.token_replacement_input}
              placeholder="Write-only replacement"
              autoComplete="new-password"
              onChange={(token_replacement_input) => onChange({ ...form, token_replacement_input })}
            />
            <ToggleField
              label="Clear stored token on save"
              checked={form.matrix.clear_access_token}
              onChange={(clear_access_token) =>
                onChange({ ...form, matrix: { ...form.matrix, clear_access_token } })
              }
            />
          </div>
        </div>
      </div>
      <FieldMessage tone="info">
        Leaving the replacement field blank preserves the stored token. The current token is never
        loaded into the browser.
      </FieldMessage>
    </section>
  )
}
