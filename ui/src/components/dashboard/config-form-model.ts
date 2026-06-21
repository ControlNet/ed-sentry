import type {
  EditableConfigUpdate,
  EditableConfigView,
  LogLevelKey,
  MatrixConfigView,
} from "@/adapters/config"

export type ConfigFormState = EditableConfigUpdate & {
  readonly token_replacement_input: string
}

const defaultMatrix: MatrixConfigView = {
  enabled: false,
  homeserver: null,
  user_id: null,
  room_id: null,
  mention_user_id: null,
  status_update_interval_seconds: 60,
  access_token_present: false,
}

export function formFromConfig(config: EditableConfigView): ConfigFormState {
  const matrix = config.matrix ?? defaultMatrix
  return {
    journal: {
      folder: config.journal.folder,
      recent_files: config.journal.recent_files,
    },
    monitor: config.monitor,
    log_levels: copyLogLevels(config.log_levels),
    matrix: {
      enabled: matrix.enabled,
      homeserver: matrix.homeserver ?? null,
      user_id: matrix.user_id ?? null,
      room_id: matrix.room_id ?? null,
      mention_user_id: matrix.mention_user_id ?? null,
      status_update_interval_seconds: matrix.status_update_interval_seconds,
      access_token_replacement: null,
      clear_access_token: false,
    },
    web: {
      enabled: config.web.enabled,
      host: config.web.host,
      port: config.web.port,
      open_browser: config.web.open_browser,
    },
    token_replacement_input: "",
  }
}

export function updateFromForm(form: ConfigFormState): EditableConfigUpdate {
  return {
    journal: {
      folder: form.journal.folder.trim(),
      recent_files: form.journal.recent_files,
    },
    monitor: form.monitor,
    log_levels: form.log_levels,
    matrix: {
      enabled: form.matrix.enabled,
      homeserver: nullableTrimmed(form.matrix.homeserver),
      user_id: nullableTrimmed(form.matrix.user_id),
      room_id: nullableTrimmed(form.matrix.room_id),
      mention_user_id: nullableTrimmed(form.matrix.mention_user_id),
      status_update_interval_seconds: form.matrix.status_update_interval_seconds,
      access_token_replacement: nullableTrimmed(form.token_replacement_input),
      clear_access_token: form.matrix.clear_access_token,
    },
    web: {
      enabled: form.web.enabled,
      host: form.web.host.trim(),
      port: form.web.port,
      open_browser: form.web.open_browser,
    },
  }
}

export function isConfigFormDirty(form: ConfigFormState, saved: ConfigFormState): boolean {
  return JSON.stringify(updateFromForm(form)) !== JSON.stringify(updateFromForm(saved))
}

export function validateConfigForm(form: ConfigFormState): readonly string[] {
  const errors: string[] = []
  if (form.journal.folder.trim().length === 0) {
    errors.push("Journal folder is required.")
  }
  addRangeError(errors, "Recent Journal files", form.journal.recent_files, 1, 100)
  addRangeError(errors, "Warn kill rate", form.monitor.warn_kill_rate, 0, 1000)
  addRangeError(errors, "Duplicate suppression max", form.monitor.duplicate_max, 0, 1000)
  addRangeError(errors, "Minimum scan level", form.monitor.min_scan_level, 0, 3)
  addRangeError(errors, "Poll interval", form.monitor.poll_interval_ms, 100, 60_000)
  addRangeError(errors, "Web port", form.web.port, 1, 65_535)
  addRangeError(
    errors,
    "Matrix status cadence",
    form.matrix.status_update_interval_seconds,
    10,
    86_400,
  )
  for (const [key, value] of Object.entries(form.log_levels)) {
    addRangeError(errors, logLevelLabel(key), value, 0, 5)
  }
  if (form.web.host.trim().length === 0) {
    errors.push("Web host is required.")
  }
  if (form.matrix.clear_access_token && form.token_replacement_input.trim().length > 0) {
    errors.push("Clear token and token replacement cannot be saved together.")
  }
  return errors
}

export function isLoopbackHost(host: string): boolean {
  const trimmed = host.trim()
  return (
    trimmed === "127.0.0.1" || trimmed === "localhost" || trimmed === "::1" || trimmed === "[::1]"
  )
}

function nullableTrimmed(value: string | null): string | null {
  if (value === null) {
    return null
  }
  const trimmed = value.trim()
  return trimmed.length === 0 ? null : trimmed
}

function addRangeError(
  errors: string[],
  label: string,
  value: number,
  min: number,
  max: number,
): void {
  if (!Number.isInteger(value) || value < min || value > max) {
    errors.push(`${label} must be between ${min} and ${max}.`)
  }
}

function logLevelLabel(key: string): string {
  return key.replaceAll("_", " ")
}

function copyLogLevels(logLevels: Record<LogLevelKey, number>): Record<LogLevelKey, number> {
  return {
    scan_incoming: logLevels.scan_incoming,
    scan_easy: logLevels.scan_easy,
    scan_hard: logLevels.scan_hard,
    kill_easy: logLevels.kill_easy,
    kill_hard: logLevels.kill_hard,
    fighter_hull: logLevels.fighter_hull,
    fighter_down: logLevels.fighter_down,
    ship_shields: logLevels.ship_shields,
    ship_hull: logLevels.ship_hull,
    died: logLevels.died,
    cargo_lost: logLevels.cargo_lost,
    bait_value_low: logLevels.bait_value_low,
    security_scan: logLevels.security_scan,
    security_attack: logLevels.security_attack,
    fuel_report: logLevels.fuel_report,
    fuel_low: logLevels.fuel_low,
    fuel_critical: logLevels.fuel_critical,
    missions: logLevels.missions,
    missions_all: logLevels.missions_all,
    merits: logLevels.merits,
    rank_promotion: logLevels.rank_promotion,
    no_kills: logLevels.no_kills,
    kill_rate: logLevels.kill_rate,
    summary_kills: logLevels.summary_kills,
    summary_faction: logLevels.summary_faction,
    summary_scans: logLevels.summary_scans,
    summary_bounties: logLevels.summary_bounties,
    summary_merits: logLevels.summary_merits,
    duplicate_suppression: logLevels.duplicate_suppression,
  }
}
