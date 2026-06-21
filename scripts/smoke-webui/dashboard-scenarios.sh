run_live_dashboard() {
  local bin port journal_dir journal config stdout stderr url screenshot
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/private-journal-root/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  screenshot="${evidence%.txt}.png"
  write_dashboard_journal "$journal_dir"
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  build_webui_dist
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$tmp_dir/root.http"
  wait_for_text "$stdout" 'Kill'
  {
    printf 'SCENARIO: live-dashboard\n'
    printf 'COMMAND: VITE_DASHBOARD_ADAPTER=web pnpm --dir ui build\n'
    cat "$tmp_dir/ui-build.log"
    printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url"
    printf 'COMMAND: pnpm --dir ui exec node ./scripts/axum-smoke.mjs --scenario live-dashboard --url %s --screenshot %s\n' "$url" "$screenshot"
    (cd "$repo_root/ui" && pnpm exec node ./scripts/axum-smoke.mjs --scenario live-dashboard --url "$url" --screenshot "$(repo_path "$screenshot")")
    assert_child_stopped
  } > "$evidence"
}

run_production_dashboard() {
  local bin port journal_dir journal config stdout stderr url screenshot
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/private-journal-root/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  screenshot="${evidence%.txt}.png"
  write_dashboard_journal "$journal_dir"
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  build_webui_dist
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$tmp_dir/root.http"
  wait_for_text "$stdout" 'Kill'
  {
    printf 'SCENARIO: production-dashboard\n'
    printf 'COMMAND: VITE_DASHBOARD_ADAPTER=web pnpm --dir ui build\n'
    cat "$tmp_dir/ui-build.log"
    printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url"
    printf 'COMMAND: pnpm --dir ui exec node ./scripts/axum-smoke.mjs --scenario live-dashboard --url %s --screenshot %s\n' "$url" "$screenshot"
    (cd "$repo_root/ui" && pnpm exec node ./scripts/axum-smoke.mjs --scenario live-dashboard --url "$url" --screenshot "$(repo_path "$screenshot")")
    assert_child_stopped
  } > "$evidence"
}

run_buffered_events() {
  local bin port journal_dir journal config stdout stderr url screenshot
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/private-journal-root/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  screenshot="${evidence%.txt}.png"
  write_dashboard_journal "$journal_dir"
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  build_webui_dist
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$tmp_dir/root.http"
  wait_for_text "$stdout" 'Kill'
  {
    printf 'SCENARIO: buffered-events\n'
    printf 'COMMAND: VITE_DASHBOARD_ADAPTER=web pnpm --dir ui build\n'
    cat "$tmp_dir/ui-build.log"
    printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url"
    printf 'COMMAND: pnpm --dir ui exec node ./scripts/axum-smoke.mjs --scenario buffered-events --url %s --screenshot %s --append-journal <fixture journal>\n' "$url" "$screenshot"
    (cd "$repo_root/ui" && pnpm exec node ./scripts/axum-smoke.mjs --scenario buffered-events --url "$url" --screenshot "$(repo_path "$screenshot")" --append-journal "$journal")
    assert_child_stopped
  } > "$evidence"
}

run_responsive() {
  local bin port journal_dir journal config stdout stderr url evidence_dir
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/private-journal-root/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  evidence_dir="$(dirname "$evidence")"
  write_dashboard_journal "$journal_dir"
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  build_webui_dist
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$tmp_dir/root.http"
  wait_for_text "$stdout" 'Kill'
  {
    printf 'SCENARIO: responsive\n'
    printf 'COMMAND: VITE_DASHBOARD_ADAPTER=web pnpm --dir ui build\n'
    cat "$tmp_dir/ui-build.log"
    printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url"
    printf 'COMMAND: pnpm --dir ui exec node ./scripts/axum-smoke.mjs --scenario responsive --url %s --responsive-dir %s\n' "$url" "$evidence_dir"
    (cd "$repo_root/ui" && pnpm exec node ./scripts/axum-smoke.mjs --scenario responsive --url "$url" --responsive-dir "$(repo_path "$evidence_dir")")
    assert_child_stopped
  } > "$evidence"
}
