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
CONFIG_TEMPLATE="$REPO_ROOT/config.example.toml"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

require_command cargo
require_command zip
require_command sha256sum

cd "$REPO_ROOT"

printf 'Building %s release binary...\n' "$TARGET"
cargo build --release --target "$TARGET"

if [[ ! -f "$EXE_PATH" ]]; then
    printf 'Expected binary was not produced: %s\n' "$EXE_PATH" >&2
    exit 1
fi

if [[ ! -f "$CONFIG_TEMPLATE" ]]; then
    printf 'Missing safe config template: %s\n' "$CONFIG_TEMPLATE" >&2
    exit 1
fi

STAGING_DIR=$(mktemp -d "${TMPDIR:-/tmp}/ed-sentry-windows-gnu.XXXXXX")
trap 'rm -rf "$STAGING_DIR"' EXIT

mkdir -p "$STAGING_DIR/$PACKAGE_NAME" "$DIST_DIR"
cp "$EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/${PACKAGE_NAME}.exe"
cp "$CONFIG_TEMPLATE" "$STAGING_DIR/$PACKAGE_NAME/config.toml"

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
sha256sum "$ZIP_PATH" "$EXTRACTED_DIR/${PACKAGE_NAME}.exe"
