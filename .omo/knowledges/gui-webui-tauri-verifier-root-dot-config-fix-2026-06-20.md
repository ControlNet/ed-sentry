# GUI/WebUI/Tauri Verifier Root Dot-Config Fix

Date: 2026-06-21 UTC

The focused verifier `scripts/verify-gui-webui-tauri.sh` now includes the changed root dot-configuration files `.github/workflows/release.yml` and `.gitignore` in the privacy/secret scan roots when those files exist.

This is intentionally narrow. The scan continues to include the prior GUI/WebUI/Tauri surface through `src`, `tests`, `scripts`, `ui`, root config/docs, and packaging files. The existing generated/heavy/evidence/git exclusions remain in place, including `.git/**`, `.omo/evidence/**`, `target/**`, `ui/node_modules/**`, `ui/dist/**`, Playwright outputs, Tauri icons, and Tauri target output.

The raw match suppression behavior remains unchanged: ripgrep writes matches to a temporary file, sanitized evidence reports only counts/categories, and any unexpected secret-like match line fails the verifier.

Verification artifacts for this fix use the prefix:

- `.omo/evidence/gui-webui-tauri/global-review-code-quality-final-rerun-fix2-*`
