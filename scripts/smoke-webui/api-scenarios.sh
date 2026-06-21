run_root() {
  local bin port journal_dir journal config stdout stderr url
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'ed-sentry WebUI root smoke'
  write_config "$config" "$journal_dir" "$port"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app_with_status_feed "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$evidence"
  printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url" >> "$evidence"
  assert_child_stopped >> "$evidence"
  printf 'ED_SENTRY_WEBUI_URL=%s\n' "$url"
}

run_packaged_assets() {
  local source_bin package_bin port journal_dir journal config stdout stderr url
  source_bin="$(binary_path)"
  mkdir -p "$tmp_dir/package"
  package_bin="$tmp_dir/package/ed-sentry"
  cp "$source_bin" "$package_bin"
  chmod +x "$package_bin"
  write_dist "$tmp_dir/package/webui" 'packaged sibling WebUI root smoke'
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_config "$config" "$journal_dir" "$port"
  unset ED_SENTRY_WEBUI_DIST
  start_app "$package_bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_http "$url" "$evidence"
  printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url" >> "$evidence"
  assert_child_stopped >> "$evidence"
  printf 'ED_SENTRY_WEBUI_URL=%s\n' "$url"
}

run_occupied_port() {
  local bin port_file port journal_dir journal config stdout stderr
  bin="$(binary_path)"
  port_file="$tmp_dir/occupied.port"
  start_port_holder "$port_file"
  port="$(cat "$port_file")"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'occupied port WebUI smoke'
  write_config "$config" "$journal_dir" "$port"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app_with_status_feed "$bin" "$config" "$journal" "$stdout" "$stderr"
  wait_for_text "$stderr" 'Warning: WebUI bind failed'
  wait_for_text "$stdout" 'Scan: Smoke Raider'
  {
    printf 'SCENARIO: occupied-port\n'
    printf 'PORT: %s\n' "$port"
    printf '\nSTDOUT:\n'
    cat "$stdout"
    printf '\nSTDERR:\n'
    cat "$stderr"
    printf '\n'
    assert_child_stopped
  } > "$evidence"
}

run_snapshot() {
  local bin port journal_dir journal config stdout stderr url
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'snapshot api smoke'
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app_with_status_feed "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_text "$stdout" 'Scan: Smoke Raider'
  wait_for_api "$url" "/api/snapshot" "$evidence"
  grep -q '"session"' "$evidence"
  grep -q '"missions"' "$evidence"
  grep -q '"events"' "$evidence"
  printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url" >> "$evidence"
  assert_child_stopped >> "$evidence"
}

run_config_redaction() {
  local bin port journal_dir journal config stdout stderr url
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'config redaction api smoke'
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_api "$url" "/api/config" "$evidence"
  if grep -q 'fixture-smoke-access-token' "$evidence"; then
    printf 'config response leaked fixture access token\n' >&2
    return 1
  fi
  grep -q '"access_token_present":true' "$evidence"
  printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url" >> "$evidence"
  assert_child_stopped >> "$evidence"
}

run_non_loopback_config_write() {
  local bin port journal_dir journal config stdout stderr url body
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  body='{"web":{"enabled":true,"host":"127.0.0.1","port":8765,"open_browser":false}}'
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'non loopback write api smoke'
  write_config_with_host "$config" "$journal_dir" "$port" "0.0.0.0"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_api "$url" "/api/health" "$tmp_dir/health.http"
  curl --silent --show-error --http1.1 --include \
    -X PUT \
    -H 'Host: 0.0.0.0' \
    -H 'Content-Type: application/json' \
    --data "$body" \
    "$url/api/config" > "$evidence"
  grep -q 'HTTP/1.1 403 Forbidden' "$evidence"
  printf '\nED_SENTRY_WEBUI_URL=%s\n' "$url" >> "$evidence"
  assert_child_stopped >> "$evidence"
}

run_websocket() {
  local bin port journal_dir journal config stdout stderr url ws_url
  if [[ -z "$probe" ]]; then
    printf '%s\n' "$usage_text" >&2
    exit 2
  fi
  bin="$(binary_path)"
  port="$(choose_port)"
  journal_dir="$tmp_dir/journal"
  journal="$journal_dir/Journal.2035-01-03T100000.01.log"
  config="$tmp_dir/config.toml"
  stdout="$tmp_dir/stdout.log"
  stderr="$tmp_dir/stderr.log"
  write_journal "$journal_dir"
  write_dist "$tmp_dir/dist" 'websocket api smoke'
  write_config_with_host "$config" "$journal_dir" "$port" "127.0.0.1"
  export ED_SENTRY_WEBUI_DIST="$tmp_dir/dist"
  start_app "$bin" "$config" "$journal" "$stdout" "$stderr"
  url="http://127.0.0.1:$port"
  wait_for_api "$url" "/api/health" "$tmp_dir/health.http"
  wait_for_text "$stdout" 'Scan: Smoke Raider'
  ws_url="ws://127.0.0.1:$port/api/events"
  node "$probe" "$ws_url" > "$evidence"
  grep -q '"type":"hello"' "$evidence"
  grep -q '"snapshot"' "$evidence"
  grep -q '"event_feed"' "$evidence"
  printf '\n{"url":"%s"}\n' "$ws_url" >> "$evidence"
  assert_child_stopped >> "$evidence"
}
