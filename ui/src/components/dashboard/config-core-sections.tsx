import {
  ConfigSection,
  FieldMessage,
  NumberField,
  SelectField,
  TextField,
  ToggleField,
} from "./config-form-fields"
import type { ConfigFormState } from "./config-form-model"
import { Ind } from "./g1-ui"

type ConfigSectionProps = {
  readonly form: ConfigFormState
  readonly onChange: (form: ConfigFormState) => void
}

const supportedTunnelProviders = [
  { value: "cloudflare_quick", label: "Cloudflare Quick Tunnel" },
] as const

export function JournalConfigSection({ form, onChange }: ConfigSectionProps): React.JSX.Element {
  return (
    <ConfigSection id="sys-journal" label="Journal settings" title="Journal">
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
    </ConfigSection>
  )
}

export function MatrixConfigSection({
  form,
  tokenPresent,
  onChange,
}: ConfigSectionProps & { readonly tokenPresent: boolean }): React.JSX.Element {
  return (
    <ConfigSection id="sys-matrix" label="Matrix settings" title="Matrix">
      <p style={{ margin: "0 0 6px" }} data-testid="matrix-token-state">
        <Ind tone={tokenPresent ? "ok" : "off"}>
          {tokenPresent ? "Access token stored" : "No access token stored"}
        </Ind>
      </p>
      <ToggleField
        label="Enable Matrix"
        checked={form.matrix.enabled}
        onChange={(enabled) => onChange({ ...form, matrix: { ...form.matrix, enabled } })}
      />
      <TextField
        label="Homeserver URI"
        value={form.matrix.homeserver ?? ""}
        onChange={(homeserver) => onChange({ ...form, matrix: { ...form.matrix, homeserver } })}
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
        label="Status update interval (seconds)"
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
        placeholder="New token"
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
      <FieldMessage tone="info">
        Leave blank to keep the stored token. The stored token is never sent to the browser.
      </FieldMessage>
    </ConfigSection>
  )
}

export function TunnelConfigSection({
  form,
  passwordPresent,
  onChange,
}: ConfigSectionProps & { readonly passwordPresent: boolean }): React.JSX.Element {
  const providerOptions = supportedTunnelProviders.some(
    (option) => option.value === form.tunnel.provider,
  )
    ? supportedTunnelProviders
    : [...supportedTunnelProviders, { value: form.tunnel.provider, label: form.tunnel.provider }]
  return (
    <ConfigSection id="sys-tunnel" label="Tunnel settings" title="Tunnel">
      <p style={{ margin: "0 0 6px" }}>
        <Ind tone={passwordPresent ? "ok" : "off"}>
          {passwordPresent ? "Config password stored" : "No config password stored"}
        </Ind>
      </p>
      <SelectField
        label="Provider"
        options={providerOptions}
        value={form.tunnel.provider}
        onChange={(provider) => onChange({ ...form, tunnel: { ...form.tunnel, provider } })}
      />
      <ToggleField
        label="Auto start tunnel"
        checked={form.tunnel.auto_start}
        onChange={(auto_start) => onChange({ ...form, tunnel: { ...form.tunnel, auto_start } })}
      />
      <TextField
        label="Replace tunnel config password"
        type="password"
        value={form.tunnel_password_replacement_input}
        placeholder="New password"
        autoComplete="new-password"
        onChange={(tunnel_password_replacement_input) =>
          onChange({ ...form, tunnel_password_replacement_input })
        }
      />
      <ToggleField
        label="Clear tunnel config password on save"
        checked={form.tunnel.clear_config_password}
        onChange={(clear_config_password) =>
          onChange({ ...form, tunnel: { ...form.tunnel, clear_config_password } })
        }
      />
      <FieldMessage tone="info">
        Leave blank to keep the stored password. The stored password is never sent to the browser.
      </FieldMessage>
    </ConfigSection>
  )
}

export function WebConfigSection({ form, onChange }: ConfigSectionProps): React.JSX.Element {
  return (
    <ConfigSection id="sys-web" label="Web settings" title="Web UI">
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
    </ConfigSection>
  )
}
