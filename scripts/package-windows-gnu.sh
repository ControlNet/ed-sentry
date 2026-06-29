#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

TARGET="x86_64-pc-windows-gnu"
PACKAGE_NAME="ed-sentry"
CORE_PACKAGE_NAME="ed-sentry-core"
DIST_DIR="$REPO_ROOT/dist"
EXTRACTED_DIR="$DIST_DIR/$PACKAGE_NAME"
ZIP_PATH="$DIST_DIR/${PACKAGE_NAME}-${TARGET}.zip"
CORE_EXE_PATH="$REPO_ROOT/target/$TARGET/release/${CORE_PACKAGE_NAME}.exe"
GUI_EXE_PATH="$REPO_ROOT/ui/src-tauri/target/$TARGET/release/${PACKAGE_NAME}.exe"
CONFIG_TEMPLATE="$REPO_ROOT/config.example.toml"
WEBUI_DIST="$REPO_ROOT/ui/dist"
WEBVIEW2_LOADER_PATH=""
CLOUDFLARED_CACHE_TEST_ROOT=""
CLOUDFLARED_URL_FILE="$SCRIPT_DIR/cloudflared-windows-amd64.url"
CLOUDFLARED_SHA256_FILE="$SCRIPT_DIR/cloudflared-windows-amd64.sha256"
CLOUDFLARED_LICENSE_PATH="$REPO_ROOT/third_party/cloudflared/LICENSE.txt"
CLOUDFLARED_CACHE_DIR="$REPO_ROOT/target/cloudflared-cache"
CLOUDFLARED_CACHE_PATH="$CLOUDFLARED_CACHE_DIR/cloudflared-windows-amd64.exe"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

read_first_line() {
    local path="$1"
    local label="$2"
    local value=""

    if [[ ! -f "$path" ]]; then
        printf 'Missing %s metadata: %s\n' "$label" "$path" >&2
        return 1
    fi
    IFS= read -r value <"$path" || true
    value=${value%$'\r'}
    if [[ -z "$value" ]]; then
        printf 'Empty %s metadata: %s\n' "$label" "$path" >&2
        return 1
    fi
    printf '%s' "$value"
}

sha256_hex() {
    local path="$1"
    local output=""

    output=$(sha256sum "$path")
    printf '%s' "${output%% *}"
}

ensure_cloudflared_cached() {
    local url=""
    local expected_hash=""
    local actual_hash=""
    local download_path=""

    url=$(read_first_line "$CLOUDFLARED_URL_FILE" "cloudflared URL") || return 1
    expected_hash=$(read_first_line "$CLOUDFLARED_SHA256_FILE" "cloudflared SHA-256") || return 1
    if [[ ! "$expected_hash" =~ ^[0-9a-fA-F]{64}$ ]]; then
        printf 'Invalid cloudflared SHA-256 metadata: %s\n' "$expected_hash" >&2
        return 1
    fi

    if [[ -f "$CLOUDFLARED_CACHE_PATH" ]]; then
        actual_hash=$(sha256_hex "$CLOUDFLARED_CACHE_PATH")
        if [[ "$actual_hash" == "$expected_hash" ]]; then
            printf 'Using verified cloudflared cache: %s\n' "$CLOUDFLARED_CACHE_PATH"
            return 0
        fi
        printf 'Cached cloudflared checksum mismatch: expected %s, got %s\n' "$expected_hash" "$actual_hash" >&2
    fi

    require_command curl
    mkdir -p "$CLOUDFLARED_CACHE_DIR"
    download_path=$(mktemp "$CLOUDFLARED_CACHE_DIR/cloudflared-windows-amd64.exe.XXXXXX")
    printf 'Downloading cloudflared from pinned URL: %s\n' "$url"
    if ! curl -fL --retry 2 --connect-timeout 15 --output "$download_path" "$url"; then
        rm -f "$download_path"
        printf 'No verified cloudflared cache is available and download failed.\n' >&2
        return 1
    fi

    actual_hash=$(sha256_hex "$download_path")
    if [[ "$actual_hash" != "$expected_hash" ]]; then
        rm -f "$download_path"
        printf 'Downloaded cloudflared checksum mismatch: expected %s, got %s\n' "$expected_hash" "$actual_hash" >&2
        return 1
    fi
    mv "$download_path" "$CLOUDFLARED_CACHE_PATH"
    printf 'Cached verified cloudflared: %s\n' "$CLOUDFLARED_CACHE_PATH"
}

stage_cloudflared() {
    local stage_dir="$STAGING_DIR/$PACKAGE_NAME/tools/cloudflared"

    ensure_cloudflared_cached
    if [[ ! -f "$CLOUDFLARED_LICENSE_PATH" ]]; then
        printf 'Missing cloudflared license: %s\n' "$CLOUDFLARED_LICENSE_PATH" >&2
        exit 1
    fi
    mkdir -p "$stage_dir"
    cp "$CLOUDFLARED_CACHE_PATH" "$stage_dir/cloudflared.exe"
    cp "$CLOUDFLARED_LICENSE_PATH" "$stage_dir/LICENSE-cloudflared.txt"
}

run_cloudflared_cache_tests() {
    local test_root=""
    local source_path=""
    local expected_hash=""
    local error_log=""

    require_command curl
    test_root=$(mktemp -d "${TMPDIR:-/tmp}/ed-sentry-cloudflared-cache-test.XXXXXX")
    CLOUDFLARED_CACHE_TEST_ROOT="$test_root"
    trap 'rm -rf "$CLOUDFLARED_CACHE_TEST_ROOT"' EXIT
    source_path="$test_root/source-cloudflared.exe"
    CLOUDFLARED_URL_FILE="$test_root/cloudflared.url"
    CLOUDFLARED_SHA256_FILE="$test_root/cloudflared.sha256"
    CLOUDFLARED_CACHE_DIR="$test_root/cache"
    CLOUDFLARED_CACHE_PATH="$CLOUDFLARED_CACHE_DIR/cloudflared-windows-amd64.exe"
    error_log="$test_root/error.log"

    printf 'fixture-v1' >"$source_path"
    expected_hash=$(sha256_hex "$source_path")
    printf 'file://%s\n' "$source_path" >"$CLOUDFLARED_URL_FILE"
    printf '%s\n' "$expected_hash" >"$CLOUDFLARED_SHA256_FILE"
    ensure_cloudflared_cached

    printf 'file://%s/missing.exe\n' "$test_root" >"$CLOUDFLARED_URL_FILE"
    ensure_cloudflared_cached

    printf 'fixture-v2' >"$source_path"
    expected_hash=$(sha256_hex "$source_path")
    printf 'file://%s\n' "$source_path" >"$CLOUDFLARED_URL_FILE"
    printf '%s\n' "$expected_hash" >"$CLOUDFLARED_SHA256_FILE"
    printf 'corrupt-cache' >"$CLOUDFLARED_CACHE_PATH"
    ensure_cloudflared_cached

    printf 'corrupt-cache' >"$CLOUDFLARED_CACHE_PATH"
    printf 'file://%s/missing.exe\n' "$test_root" >"$CLOUDFLARED_URL_FILE"
    if ensure_cloudflared_cached 2>"$error_log"; then
        printf 'Expected invalid cache plus missing download to fail.\n' >&2
        return 1
    fi
    if [[ $(<"$error_log") != *"No verified cloudflared cache is available"* ]]; then
        printf 'Expected missing network/cache failure message.\n' >&2
        return 1
    fi

    printf 'cloudflared cache tests passed\n'
}

if [[ "${1:-}" == "--test-cloudflared-cache" ]]; then
    require_command sha256sum
    run_cloudflared_cache_tests
    exit 0
fi

if [[ $# -gt 0 ]]; then
    printf 'Usage: %s [--test-cloudflared-cache]\n' "$0" >&2
    exit 2
fi

require_command cargo
require_command pnpm
require_command zip
require_command sha256sum

cd "$REPO_ROOT"

printf 'Building WebUI assets...\n'
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build

printf 'Building %s release binary with desktop GUI mode...\n' "$TARGET"
cargo build --release --target "$TARGET"

printf 'Building %s tiny GUI launcher...\n' "$TARGET"
cargo build --manifest-path "$REPO_ROOT/ui/src-tauri/Cargo.toml" --release --target "$TARGET"

for candidate in "$REPO_ROOT"/target/$TARGET/release/build/webview2-com-sys-*/out/x64/WebView2Loader.dll; do
    if [[ -f "$candidate" ]]; then
        WEBVIEW2_LOADER_PATH="$candidate"
        break
    fi
done

if [[ ! -f "$CORE_EXE_PATH" ]]; then
    printf 'Expected core binary was not produced: %s\n' "$CORE_EXE_PATH" >&2
    exit 1
fi

if [[ ! -f "$GUI_EXE_PATH" ]]; then
    printf 'Expected GUI binary was not produced: %s\n' "$GUI_EXE_PATH" >&2
    exit 1
fi

if [[ -z "$WEBVIEW2_LOADER_PATH" ]]; then
    printf 'Expected WebView2 loader DLL was not produced: %s\n' "$WEBVIEW2_LOADER_PATH" >&2
    exit 1
fi

if [[ ! -f "$CONFIG_TEMPLATE" ]]; then
    printf 'Missing safe config template: %s\n' "$CONFIG_TEMPLATE" >&2
    exit 1
fi

if [[ ! -f "$WEBUI_DIST/index.html" ]]; then
    printf 'Missing built WebUI index: %s\n' "$WEBUI_DIST/index.html" >&2
    exit 1
fi

STAGING_DIR=$(mktemp -d "${TMPDIR:-/tmp}/ed-sentry-windows-gnu.XXXXXX")
trap 'rm -rf "$STAGING_DIR"' EXIT

mkdir -p "$STAGING_DIR/$PACKAGE_NAME/webui" "$DIST_DIR"
cp "$GUI_EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/${PACKAGE_NAME}.exe"
cp "$CORE_EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/${CORE_PACKAGE_NAME}.exe"
cp "$WEBVIEW2_LOADER_PATH" "$STAGING_DIR/$PACKAGE_NAME/WebView2Loader.dll"
cp "$CONFIG_TEMPLATE" "$STAGING_DIR/$PACKAGE_NAME/config.toml"
cp -R "$WEBUI_DIST"/. "$STAGING_DIR/$PACKAGE_NAME/webui/"
stage_cloudflared

case "$EXTRACTED_DIR" in
    "$REPO_ROOT"/dist/ed-sentry)
        rm -rf "$EXTRACTED_DIR"
        ;;
    *)
        printf 'Refusing to remove unexpected extracted directory: %s\n' "$EXTRACTED_DIR" >&2
        exit 1
        ;;
esac
cp -R "$STAGING_DIR/$PACKAGE_NAME" "$EXTRACTED_DIR"

ZIP_TMP="$STAGING_DIR/${PACKAGE_NAME}-${TARGET}.zip"
(
    cd "$STAGING_DIR"
    zip -qr "$ZIP_TMP" "$PACKAGE_NAME"
)
mv "$ZIP_TMP" "$ZIP_PATH"

printf 'Packaged Windows GNU artifact:\n'
printf '  %s\n' "$ZIP_PATH"
printf '  %s\n' "$EXTRACTED_DIR"
printf '  WebUI: %s\n' "$EXTRACTED_DIR/webui"
test -f "$EXTRACTED_DIR/webui/index.html"
sha256sum "$ZIP_PATH" "$EXTRACTED_DIR/${PACKAGE_NAME}.exe" "$EXTRACTED_DIR/${CORE_PACKAGE_NAME}.exe" "$EXTRACTED_DIR/webui/index.html"
sha256sum "$EXTRACTED_DIR/WebView2Loader.dll" "$EXTRACTED_DIR/tools/cloudflared/cloudflared.exe"
