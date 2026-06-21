#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
evidence_dir="$repo_root/.omo/evidence/gui-webui-tauri"
mkdir -p "$evidence_dir"

section() {
  printf '\n== %s ==\n' "$1"
}

run() {
  printf '+ %q' "$1"
  shift
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

run_smoke() {
  local scenario="$1"
  local evidence="$evidence_dir/task-15-full-gate-${scenario}.txt"
  local args=("$repo_root/scripts/smoke-webui.sh" --scenario "$scenario" --evidence "$evidence")
  if [[ "$scenario" == "websocket" ]]; then
    args+=(--probe "$repo_root/scripts/probe-websocket.mjs")
  fi
  run "$repo_root" "${args[@]}"
  printf 'EVIDENCE: %s\n' "$evidence"
}

run_secret_scan() {
  local secret_evidence="$evidence_dir/task-15-full-gate-secret-grep.txt"
  local secret_matches
  secret_matches="$(mktemp)"
  local secret_pattern='fixture-smoke-access-token|Authorization:[[:space:]]*Bearer[[:space:]]+[A-Za-z0-9._~+/=-]{16,}|-----BEGIN (RSA |OPENSSH |EC |DSA )?PRIVATE KEY|sk-[A-Za-z0-9_-]{40,}|xox[baprs]-[A-Za-z0-9-]+'
  local secret_scan_roots=()
  local candidate
  for candidate in \
    src tests scripts ui \
    .github/workflows/release.yml .gitignore \
    Cargo.toml Cargo.lock README.md DESIGN.md config.example.toml docs \
    package.json rust-toolchain.toml rustfmt.toml
  do
    if [[ -e "$candidate" ]]; then
      secret_scan_roots+=("$candidate")
    fi
  done

  if ((${#secret_scan_roots[@]} == 0)); then
    printf 'SECRET_SCAN: no scan roots found\n' >&2
    exit 1
  fi

  set +e
  rg -n -I --regexp "$secret_pattern" \
    --glob '!target/**' \
    --glob '!ui/node_modules/**' \
    --glob '!ui/dist/**' \
    --glob '!ui/playwright-report/**' \
    --glob '!ui/test-results/**' \
    --glob '!ui/src-tauri/icons/**' \
    --glob '!ui/src-tauri/target/**' \
    --glob '!.omo/evidence/**' \
    --glob '!.git/**' \
    -- "${secret_scan_roots[@]}" >"$secret_matches"
  local secret_scan_status=$?
  set -e

  if ((secret_scan_status != 0 && secret_scan_status != 1)); then
    printf 'SECRET_SCAN: scanner failed with status %s\n' "$secret_scan_status" >&2
    rm -f "$secret_matches"
    exit "$secret_scan_status"
  fi

  local total_matches allowed_fixture_matches unexpected_matches
  local bearer_matches private_key_matches openai_key_matches slack_token_matches
  local untracked_scanned_files
  total_matches="$(wc -l <"$secret_matches" | tr -d ' ')"
  allowed_fixture_matches="$(awk '/fixture-smoke-access-token/ { count++ } END { print count + 0 }' "$secret_matches")"
  unexpected_matches="$(awk '!/fixture-smoke-access-token/ { count++ } END { print count + 0 }' "$secret_matches")"
  bearer_matches="$(awk '!/fixture-smoke-access-token/ && /Authorization:/ { count++ } END { print count + 0 }' "$secret_matches")"
  private_key_matches="$(awk '!/fixture-smoke-access-token/ && /PRIVATE KEY/ { count++ } END { print count + 0 }' "$secret_matches")"
  openai_key_matches="$(awk '!/fixture-smoke-access-token/ && /sk-/ { count++ } END { print count + 0 }' "$secret_matches")"
  slack_token_matches="$(awk '!/fixture-smoke-access-token/ && /xox[baprs]-/ { count++ } END { print count + 0 }' "$secret_matches")"
  untracked_scanned_files="$(git ls-files --others --exclude-standard -- "${secret_scan_roots[@]}" | wc -l | tr -d ' ')"

  {
    printf 'SECRET_SCAN_SCOPE: working-tree roots scanned with rg, including untracked non-ignored files\n'
    printf 'SECRET_SCAN_ROOTS_COUNT: %s\n' "${#secret_scan_roots[@]}"
    printf 'SECRET_SCAN_UNTRACKED_FILES_IN_SCOPE: %s\n' "$untracked_scanned_files"
    printf 'SECRET_SCAN_TOTAL_MATCH_LINES: %s\n' "$total_matches"
    printf 'SECRET_SCAN_ALLOWED_FIXTURE_LINES: %s\n' "$allowed_fixture_matches"
    printf 'SECRET_SCAN_UNEXPECTED_LINES: %s\n' "$unexpected_matches"
    printf 'SECRET_SCAN_UNEXPECTED_CATEGORIES: bearer=%s private_key=%s openai=%s slack=%s\n' \
      "$bearer_matches" "$private_key_matches" "$openai_key_matches" "$slack_token_matches"
  } >"$secret_evidence"

  if ((unexpected_matches > 0)); then
    cat "$secret_evidence"
    printf 'SECRET_SCAN: unexpected sensitive pattern; raw matches suppressed\n' >&2
    rm -f "$secret_matches"
    exit 1
  fi

  cat "$secret_evidence"
  printf 'SECRET_SCAN: pass; working-tree scan found no unexpected sensitive patterns\n'
  printf 'EVIDENCE: .omo/evidence/gui-webui-tauri/task-15-full-gate-secret-grep.txt\n'
  rm -f "$secret_matches"
}

cd "$repo_root"

if [[ "${VERIFY_GUI_WEBUI_TAURI_ONLY_SECRET_SCAN:-0}" == "1" ]]; then
  section "privacy and secret guard"
  run_secret_scan
  exit 0
fi

section "self-check"
run "$repo_root" test -x scripts/verify-gui-webui-tauri.sh

section "rust formatting"
run "$repo_root" cargo fmt --check

section "rust tests"
run "$repo_root" cargo test --all

section "rust clippy"
run "$repo_root" cargo clippy --all-targets --all-features -- -D warnings

section "frontend install"
run "$repo_root" pnpm --dir ui install --frozen-lockfile

section "frontend typecheck"
run "$repo_root" pnpm --dir ui typecheck

section "frontend build"
run "$repo_root" pnpm --dir ui build

section "playwright e2e"
run "$repo_root" pnpm --dir ui test:e2e -- --project=chromium

section "webui smoke"
run_smoke root
run_smoke packaged-assets
run_smoke occupied-port
run_smoke snapshot
run_smoke config-redaction
run_smoke non-loopback-config-write
run_smoke websocket
run_smoke production-dashboard
run_smoke buffered-events
run_smoke responsive

section "tauri build"
tauri_log="$evidence_dir/task-15-full-gate-tauri-build.txt"
if pnpm --dir ui tauri build >"$tauri_log" 2>&1; then
  cat "$tauri_log"
  printf 'TAURI_BUILD: pass\n'
else
  cat "$tauri_log"
  if rg -i 'webkit2gtk|javascriptcoregtk|libsoup|gtk|ayatana|appindicator|pkg-config|system library|failed to run custom build command' "$tauri_log" >/dev/null 2>&1; then
    printf 'TAURI_BUILD: environment-blocker\n'
    printf 'TAURI_BUILD_BLOCKER_LOG: %s\n' "$tauri_log"
  else
    printf 'TAURI_BUILD: fail\n' >&2
    exit 1
  fi
fi

section "privacy and secret guard"
run_secret_scan

section "complete"
printf 'VERIFY_GUI_WEBUI_TAURI: pass\n'
