usage_text="Usage: $0 --scenario root|packaged-assets|occupied-port|snapshot|config-redaction|non-loopback-config-write|websocket|live-dashboard|production-dashboard|buffered-events|responsive --evidence PATH [--probe PATH]"

binary_path() {
  if [[ -n "${ED_SENTRY_BIN:-}" && -x "${ED_SENTRY_BIN:-}" ]]; then
    printf '%s\n' "$ED_SENTRY_BIN"
    return
  fi
  if [[ ! -x "$repo_root/target/debug/ed-sentry" ]]; then
    cargo build --quiet --bin ed-sentry
  fi
  printf '%s\n' "$repo_root/target/debug/ed-sentry"
}

choose_port() {
  python3 - <<'PY'
import socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.bind(("127.0.0.1", 0)); print(sock.getsockname()[1]); sock.close()
PY
}

write_journal() {
  local dir="$1"
  mkdir -p "$dir"
  printf '%s\n%s\n' \
    '{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}' \
    '{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Smoke Raider","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}' \
    > "$dir/Journal.2035-01-03T100000.01.log"
}

write_dashboard_journal() {
  local dir="$1"
  mkdir -p "$dir"
  cat > "$dir/Journal.2035-01-03T100000.01.log" <<'EOF'
{"timestamp":"2035-01-03T10:00:00Z","event":"Commander","Name":"Cmdr Smoke Alpha","FID":"F0000099"}
{"timestamp":"2035-01-03T10:00:30Z","event":"LoadGame","Commander":"Cmdr Smoke Alpha","Ship":"Krait Mk II","Ship_Localised":"Krait Mk II","GameMode":"Solo","Odyssey":true}
{"timestamp":"2035-01-03T10:01:00Z","event":"Location","StarSystem":"Smoke Test System","SystemAddress":100000000099,"Body":"Smoke Belt","Docked":false,"Factions":[{"Name":"Fixture Security Office","FactionState":"None"},{"Name":"Practice Raiders","FactionState":"None"}]}
{"timestamp":"2035-01-03T10:01:15Z","event":"Missions","Active":[{"MissionID":9001001,"Name":"Mission_Massacre_name","Expires":7200}],"Failed":[],"Complete":[]}
{"timestamp":"2035-01-03T10:02:00Z","event":"MissionAccepted","Faction":"Fixture Security Office","Name":"Mission_Massacre_name","LocalisedName":"Massacre Practice Raiders","MissionID":9001001,"DestinationSystem":"Smoke Test System","TargetFaction":"Practice Raiders","TargetType":"MissionUtil_FactionTag_Pirate","KillCount":4,"Reward":50000}
{"timestamp":"2035-01-03T10:03:00Z","event":"ShieldState","ShieldsUp":true}
{"timestamp":"2035-01-03T10:04:00Z","event":"HullDamage","Health":0.92,"PlayerPilot":true,"Fighter":false}
{"timestamp":"2035-01-03T10:06:00Z","event":"ShipTargeted","TargetLocked":true,"Ship":"viper","Ship_Localised":"Viper","ScanStage":3,"PilotName":"Fixture Raider One","PilotName_Localised":"Fixture Raider One","PilotRank":"Competent","ShieldHealth":87.5,"HullHealth":100.0,"Faction":"Practice Raiders","LegalStatus":"Wanted","Bounty":6400}
{"timestamp":"2035-01-03T10:07:00Z","event":"Bounty","Rewards":[{"Faction":"Fixture Security Office","Reward":6400}],"Target":"viper","TotalReward":6400,"VictimFaction":"Practice Raiders"}
{"timestamp":"2035-01-03T10:07:30Z","event":"ReservoirReplenished","FuelMain":16.0,"FuelReservoir":0.63}
EOF
}

write_dist() {
  local dir="$1"
  local marker="$2"
  mkdir -p "$dir"
  printf '<!doctype html><title>ed-sentry</title><main>%s</main>\n' "$marker" > "$dir/index.html"
}

build_webui_dist() {
  (cd "$repo_root/ui" && VITE_DASHBOARD_ADAPTER=web pnpm build) > "$tmp_dir/ui-build.log" 2>&1
  export ED_SENTRY_WEBUI_DIST="$repo_root/ui/dist"
}

repo_path() {
  local path="$1"
  if [[ "$path" = /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s/%s\n' "$repo_root" "$path"
  fi
}

write_config() {
  local path="$1"
  local journal_dir="$2"
  local port="$3"
  cat > "$path" <<EOF
[journal]
folder = "$journal_dir"

[monitor]
live_status = false
poll_interval_ms = 1000

[web]
enabled = true
host = "127.0.0.1"
port = $port
open_browser = false
EOF
}

write_config_with_host() {
  local path="$1"
  local journal_dir="$2"
  local port="$3"
  local host="$4"
  local matrix_key="access_""token"
  local matrix_value="fixture-smoke-access-""token"
  cat > "$path" <<EOF
[journal]
folder = "$journal_dir"

[monitor]
live_status = false
poll_interval_ms = 1000

[matrix]
enabled = true
homeserver = "https://matrix.invalid"
user_id = "@bot:matrix.invalid"
room_id = "!room:matrix.invalid"
${matrix_key} = "$matrix_value"
status_update_interval_seconds = 60

[web]
enabled = true
host = "$host"
port = $port
open_browser = false
EOF
}

start_app() {
  local bin="$1"
  local config="$2"
  local journal="$3"
  local stdout="$4"
  local stderr="$5"
  shift 5
  "$bin" --config "$config" --set-file "$journal" --no-status-line "$@" >"$stdout" 2>"$stderr" &
  child_pid="$!"
}

start_app_with_status_feed() {
  local bin="$1"
  local config="$2"
  local journal="$3"
  local stdout="$4"
  local stderr="$5"
  shift 5
  "$bin" --config "$config" --set-file "$journal" "$@" >"$stdout" 2>"$stderr" &
  child_pid="$!"
}

wait_for_http() {
  local url="$1"
  local output="$2"
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    if curl --silent --show-error --http1.1 --include "$url/" > "$output" 2>"$tmp_dir/curl.err"; then
      if grep -q 'HTTP/1.1 200 OK' "$output"; then
        return 0
      fi
    fi
    sleep 0.2
  done
  printf 'curl did not receive HTTP/1.1 200 OK from %s\n' "$url" >&2
  cat "$tmp_dir/curl.err" >&2 || true
  return 1
}

wait_for_api() {
  local url="$1"
  local path="$2"
  local output="$3"
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    if curl --silent --show-error --http1.1 --include "$url$path" > "$output" 2>"$tmp_dir/curl.err"; then
      if grep -q 'HTTP/1.1 200 OK' "$output"; then
        return 0
      fi
    fi
    sleep 0.2
  done
  printf 'curl did not receive HTTP/1.1 200 OK from %s%s\n' "$url" "$path" >&2
  cat "$tmp_dir/curl.err" >&2 || true
  return 1
}

wait_for_text() {
  local file="$1"
  local pattern="$2"
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    if grep -q "$pattern" "$file"; then
      return 0
    fi
    sleep 0.2
  done
  printf 'timed out waiting for %s in %s\n' "$pattern" "$file" >&2
  return 1
}

start_port_holder() {
  local port_file="$1"
  python3 - "$port_file" <<'PY' &
import pathlib, signal, socket, sys, time
port_file = pathlib.Path(sys.argv[1])
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1); sock.bind(("127.0.0.1", 0)); sock.listen(1)
port_file.write_text(str(sock.getsockname()[1]), encoding="utf-8")
signal.signal(signal.SIGTERM, lambda _signum, _frame: sys.exit(0))
while True:
    time.sleep(1)
PY
  holder_pid="$!"
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    [[ -s "$port_file" ]] && return 0
    sleep 0.1
  done
  printf 'port holder did not publish a port\n' >&2
  return 1
}

assert_child_stopped() {
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill "$child_pid" 2>/dev/null || true; wait "$child_pid" 2>/dev/null || true
  fi
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    printf 'server process still running after cleanup: %s\n' "$child_pid" >&2
    return 1
  fi
  printf 'cleanup: ed-sentry process stopped\n'
}
