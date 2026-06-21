#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$repo_root/scripts/smoke-webui/common.sh"
source "$repo_root/scripts/smoke-webui/api-scenarios.sh"
source "$repo_root/scripts/smoke-webui/dashboard-scenarios.sh"

scenario=''
evidence=''
probe=''
while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario="${2:-}"
      shift 2
      ;;
    --evidence)
      evidence="${2:-}"
      shift 2
      ;;
    --probe)
      probe="${2:-}"
      shift 2
      ;;
    -h|--help)
      printf '%s\n' "$usage_text"; exit 0
      ;;
    *)
      printf '%s\n' "$usage_text" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$scenario" || -z "$evidence" ]]; then
  printf '%s\n' "$usage_text" >&2
  exit 2
fi

mkdir -p "$(dirname "$evidence")"
tmp_dir="$(mktemp -d)"
child_pid=''
holder_pid=''

cleanup() {
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill "$child_pid" 2>/dev/null || true
    wait "$child_pid" 2>/dev/null || true
  fi
  if [[ -n "$holder_pid" ]] && kill -0 "$holder_pid" 2>/dev/null; then
    kill "$holder_pid" 2>/dev/null || true
    wait "$holder_pid" 2>/dev/null || true
  fi
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

case "$scenario" in
  root)
    run_root ;;
  packaged-assets)
    run_packaged_assets ;;
  occupied-port)
    run_occupied_port ;;
  snapshot)
    run_snapshot ;;
  config-redaction)
    run_config_redaction ;;
  non-loopback-config-write)
    run_non_loopback_config_write ;;
  websocket)
    run_websocket ;;
  live-dashboard)
    run_live_dashboard ;;
  production-dashboard)
    run_production_dashboard ;;
  buffered-events)
    run_buffered_events ;;
  responsive)
    run_responsive ;;
  *)
    printf '%s\n' "$usage_text" >&2
    exit 2
    ;;
esac
