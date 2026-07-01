# Windows launcher icon packaging

- The Windows GNU package uses `cargo build --manifest-path ui/src-tauri/Cargo.toml --release --target x86_64-pc-windows-gnu` for the tiny `ed-sentry.exe` launcher instead of `tauri build`.
- Because of that direct Cargo build, the launcher needs `ui/src-tauri/build.rs` to compile `ui/src-tauri/resources/windows.rc` with `embed-resource` so `icons/icon.ico` is embedded into the PE resource table.
- The package script checks `x86_64-w64-mingw32-objdump -x <exe>` and rejects a GUI binary whose `Resource Directory [.rsrc]` entry is missing or has size `00000000`.
