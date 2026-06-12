# Windows Saved Games default Journal path

Windows default Journal discovery must resolve the system Saved Games known folder, then append `Frontier Developments/Elite Dangerous`.

Do not derive this path from `%USERPROFILE%/Saved Games`: Windows lets users relocate Saved Games through system folder settings, and that breaks a simple string join.

Implementation note:

- `src/journal.rs` uses `SHGetKnownFolderPath(FOLDERID_SavedGames, ...)` on Windows via `windows-sys`.
- Non-Windows platforms still require an explicit `journal.folder`, `--journal`, or `--set-file`.
- `journal_folder_from_saved_games()` is the platform-neutral helper for tests and path composition.
