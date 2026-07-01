#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

PACKAGE_NAME="ed-sentry"
CORE_PACKAGE_NAME="ed-sentry-core"
VERSION=$(node "$SCRIPT_DIR/sync-release-version.mjs" --print-version)
DIST_DIR="$REPO_ROOT/dist"
EXTRACTED_DIR="$DIST_DIR/$PACKAGE_NAME"
RELEASE_ZIP_PATH="$DIST_DIR/${PACKAGE_NAME}-v${VERSION}-linux-x64.zip"
CORE_EXE_PATH="$REPO_ROOT/target/release/${CORE_PACKAGE_NAME}"
CONFIG_TEMPLATE="$REPO_ROOT/config.example.toml"
README_PATH="$REPO_ROOT/README.md"
LICENSE_PATH="$REPO_ROOT/LICENSE"
WEBUI_DIST="$REPO_ROOT/ui/dist"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Missing required command: %s\n' "$1" >&2
        exit 1
    fi
}

run_package_contract_test() {
    printf '%s\n' "$RELEASE_ZIP_PATH"
    printf '%s\n' "$PACKAGE_NAME/$CORE_PACKAGE_NAME"
    printf '%s\n' "$PACKAGE_NAME/README.md"
    printf '%s\n' "$PACKAGE_NAME/LICENSE"
    printf '%s\n' "$PACKAGE_NAME/webui/index.html"
}

if [[ "${1:-}" == "--test-package-contract" ]]; then
    run_package_contract_test
    exit 0
fi

if [[ $# -gt 0 ]]; then
    printf 'Usage: %s [--test-package-contract]\n' "$0" >&2
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

printf 'Building Linux release binary...\n'
cargo build --release

if [[ ! -f "$CORE_EXE_PATH" ]]; then
    printf 'Expected core binary was not produced: %s\n' "$CORE_EXE_PATH" >&2
    exit 1
fi

if [[ ! -f "$CONFIG_TEMPLATE" ]]; then
    printf 'Missing safe config template: %s\n' "$CONFIG_TEMPLATE" >&2
    exit 1
fi

if [[ ! -f "$README_PATH" ]]; then
    printf 'Missing README: %s\n' "$README_PATH" >&2
    exit 1
fi

if [[ ! -f "$LICENSE_PATH" ]]; then
    printf 'Missing license: %s\n' "$LICENSE_PATH" >&2
    exit 1
fi

if [[ ! -f "$WEBUI_DIST/index.html" ]]; then
    printf 'Missing built WebUI index: %s\n' "$WEBUI_DIST/index.html" >&2
    exit 1
fi

STAGING_DIR=$(mktemp -d "${TMPDIR:-/tmp}/ed-sentry-linux-x64.XXXXXX")
trap 'rm -rf "$STAGING_DIR"' EXIT

mkdir -p "$STAGING_DIR/$PACKAGE_NAME/webui" "$DIST_DIR"
cp "$CORE_EXE_PATH" "$STAGING_DIR/$PACKAGE_NAME/$CORE_PACKAGE_NAME"
cp "$CONFIG_TEMPLATE" "$STAGING_DIR/$PACKAGE_NAME/config.toml"
cp "$README_PATH" "$STAGING_DIR/$PACKAGE_NAME/README.md"
cp "$LICENSE_PATH" "$STAGING_DIR/$PACKAGE_NAME/LICENSE"
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

ZIP_TMP="$STAGING_DIR/${PACKAGE_NAME}-v${VERSION}-linux-x64.zip"
(
    cd "$STAGING_DIR"
    zip -qr "$ZIP_TMP" "$PACKAGE_NAME"
)
mv "$ZIP_TMP" "$RELEASE_ZIP_PATH"

printf 'Packaged Linux artifact:\n'
printf '  %s\n' "$RELEASE_ZIP_PATH"
printf '  %s\n' "$EXTRACTED_DIR"
printf '  WebUI: %s\n' "$EXTRACTED_DIR/webui"
test -f "$EXTRACTED_DIR/webui/index.html"
test -f "$EXTRACTED_DIR/README.md"
test -f "$EXTRACTED_DIR/LICENSE"
sha256sum "$RELEASE_ZIP_PATH" "$EXTRACTED_DIR/$CORE_PACKAGE_NAME" "$EXTRACTED_DIR/webui/index.html"
