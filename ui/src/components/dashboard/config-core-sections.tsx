import { Database, Globe2, KeyRound } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import {
  FieldMessage,
  NumberField,
  SectionTitle,
  TextField,
  ToggleField,
} from "./config-form-fields"
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
    <section aria-label="Journal settings" className="grid gap-4 rounded-md border bg-card p-4">
      <div className="flex items-start gap-3">
        <Database aria-hidden="true" className="mt-1 size-5 text-primary" />
        <SectionTitle
          title="Journal"
          description="Folder-based Journal selection. Single-file source selection remains a CLI/runtime option."
        />
      </div>
      <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_12rem]">
        <TextField
          label="Journal folder"
          value={form.journal.folder}
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
    <section aria-label="Web settings" className="grid gap-4 rounded-md border bg-card p-4">
      <div className="flex items-start gap-3">
        <Globe2 aria-hidden="true" className="mt-1 size-5 text-primary" />
        <SectionTitle
          title="Web"
          description="Local WebUI bind settings. State-changing endpoints remain loopback-only."
        />
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
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
    <section aria-label="Matrix settings" className="grid gap-4 rounded-md border bg-card p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-start gap-3">
          <KeyRound aria-hidden="true" className="mt-1 size-5 text-primary" />
          <SectionTitle
            title="Matrix"
            description="Delivery identity, room target, status cadence, and write-only token controls."
          />
        </div>
        <Badge variant={tokenPresent ? "default" : "outline"}>
          {tokenPresent ? "Token stored" : "No token stored"}
        </Badge>
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        <ToggleField
          label="Enabled"
          checked={form.matrix.enabled}
          onChange={(enabled) => onChange({ ...form, matrix: { ...form.matrix, enabled } })}
        />
        <TextField
          label="Homeserver"
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
          label="Status cadence seconds"
          value={form.matrix.status_update_interval_seconds}
          min={10}
          max={86_400}
          onChange={(status_update_interval_seconds) =>
            onChange({ ...form, matrix: { ...form.matrix, status_update_interval_seconds } })
          }
        />
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
      <FieldMessage tone="info">
        Leaving the replacement field blank preserves the stored token. The current token is never
        loaded into the browser.
      </FieldMessage>
    </section>
  )
}
