#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'Usage: %s PACKAGE_DIR EVIDENCE_PATH\n' "$0" >&2
}

if [[ $# -ne 2 ]]; then
    usage
    exit 2
fi

PACKAGE_DIR="$1"
EVIDENCE_PATH="$2"
BIN_PATH="$PACKAGE_DIR/ed-sentry-core"
WEBUI_INDEX="$PACKAGE_DIR/webui/index.html"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

require_command curl
require_command python3

if [[ ! -x "$BIN_PATH" ]]; then
    printf 'Missing packaged executable: %s\n' "$BIN_PATH" >&2
    exit 1
fi

if [[ ! -f "$WEBUI_INDEX" ]]; then
    printf 'Missing packaged WebUI index: %s\n' "$WEBUI_INDEX" >&2
    exit 1
fi

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/ed-sentry-packaged-webui.XXXXXX")
CHILD_PID=''

cleanup() {
    if [[ -n "$CHILD_PID" ]] && kill -0 "$CHILD_PID" 2>/dev/null; then
        kill "$CHILD_PID" 2>/dev/null || true
        wait "$CHILD_PID" 2>/dev/null || true
    fi
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

choose_port() {
    python3 - <<'PY'
import socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

PORT=$(choose_port)
JOURNAL_DIR="$TMP_DIR/journal"
JOURNAL_FILE="$JOURNAL_DIR/Journal.2035-01-03T100000.01.log"
CONFIG_FILE="$TMP_DIR/config.toml"
STDOUT_FILE="$TMP_DIR/stdout.log"
STDERR_FILE="$TMP_DIR/stderr.log"

mkdir -p "$JOURNAL_DIR" "$(dirname "$EVIDENCE_PATH")"
printf '%s\n%s\n' \
    '{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}' \
    '{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Package Smoke","PilotName":"Fixture Pilot","LegalStatus":"Wanted"}' \
    > "$JOURNAL_FILE"

cat > "$CONFIG_FILE" <<EOF
[journal]
folder = "$JOURNAL_DIR"

[monitor]
live_status = false

[web]
enabled = true
host = "127.0.0.1"
port = $PORT
open_browser = false
EOF

"$BIN_PATH" --config "$CONFIG_FILE" --set-file "$JOURNAL_FILE" --no-status-line >"$STDOUT_FILE" 2>"$STDERR_FILE" &
CHILD_PID="$!"

URL="http://127.0.0.1:$PORT/"
for _ in {1..80}; do
    if curl -i --http1.1 --max-time 2 "$URL" > "$EVIDENCE_PATH" 2>>"$STDERR_FILE"; then
        break
    fi
    if ! kill -0 "$CHILD_PID" 2>/dev/null; then
        printf 'Packaged ed-sentry exited before WebUI served /\n' >> "$EVIDENCE_PATH"
        cat "$STDOUT_FILE" "$STDERR_FILE" >> "$EVIDENCE_PATH"
        exit 1
    fi
    sleep 0.25
done

if ! grep -Eq 'HTTP/1\.1 200 OK|HTTP/1\.1 304 Not Modified|HTTP/1\.1 206 Partial Content' "$EVIDENCE_PATH"; then
    printf '\nExpected successful HTTP status from packaged WebUI root.\n' >> "$EVIDENCE_PATH"
    cat "$STDOUT_FILE" "$STDERR_FILE" >> "$EVIDENCE_PATH"
    exit 1
fi

printf '\nPACKAGE_DIR=%s\nWEBUI_INDEX=%s\nURL=%s\ncleanup: packaged WebUI process stopped\n' "$PACKAGE_DIR" "$WEBUI_INDEX" "$URL" >> "$EVIDENCE_PATH"
