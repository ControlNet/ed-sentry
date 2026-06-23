#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

TARGET="x86_64-pc-windows-gnu"
PACKAGE_NAME="ed-sentry"
DIST_DIR="$REPO_ROOT/dist"
EXTRACTED_DIR="$DIST_DIR/$PACKAGE_NAME"
ZIP_PATH="$DIST_DIR/${PACKAGE_NAME}-${TARGET}.zip"
EXE_PATH="$REPO_ROOT/target/$TARGET/release/${PACKAGE_NAME}.exe"
GUI_EXE_PATH="$REPO_ROOT/ui/src-tauri/target/$TARGET/release/${PACKAGE_NAME}-gui.exe"
CONFIG_TEMPLATE="$REPO_ROOT/config.example.toml"
WEBUI_DIST="$REPO_ROOT/ui/dist"
WEBVIEW2_LOADER_PATH=""

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

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

if [[ ! -f "$EXE_PATH" ]]; then
    printf 'Expected binary was not produced: %s\n' "$EXE_PATH" >&2
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
cp "$EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/${PACKAGE_NAME}.exe"
cp "$GUI_EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/${PACKAGE_NAME}-gui.exe"
cp "$WEBVIEW2_LOADER_PATH" "$STAGING_DIR/$PACKAGE_NAME/WebView2Loader.dll"
cp "$CONFIG_TEMPLATE" "$STAGING_DIR/$PACKAGE_NAME/config.toml"
cp -R "$WEBUI_DIST"/. "$STAGING_DIR/$PACKAGE_NAME/webui/"

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
sha256sum "$ZIP_PATH" "$EXTRACTED_DIR/${PACKAGE_NAME}.exe" "$EXTRACTED_DIR/webui/index.html"
sha256sum "$EXTRACTED_DIR/${PACKAGE_NAME}-gui.exe" "$EXTRACTED_DIR/WebView2Loader.dll"
