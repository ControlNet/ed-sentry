Windows GNU packaging prerequisites

- Local cross-build target: `x86_64-pc-windows-gnu`.
- Required system toolchain on Ubuntu: `mingw-w64`, which provides `x86_64-w64-mingw32-dlltool` and the linker tools needed by crates that depend on `windows-sys`.
- If `cargo build --release --target x86_64-pc-windows-gnu` fails with `error calling dlltool 'x86_64-w64-mingw32-dlltool': No such file or directory`, install the toolchain with:

```bash
sudo apt-get update
sudo apt-get install -y mingw-w64
```

- After any repo change, rebuild and refresh both Linux and Windows artifacts before reporting completion.
- Use `scripts/package-windows-gnu.sh` for local Windows GNU packaging. It runs `cargo build --release --target x86_64-pc-windows-gnu`, stages `ed-sentry/ed-sentry.exe` plus `config.example.toml` as `ed-sentry/config.toml`, refreshes `dist/ed-sentry/`, and writes `dist/ed-sentry-x86_64-pc-windows-gnu.zip`.
- Do not hand-copy `dist/ed-sentry/config.toml`; stale extracted dist contents caused the packaged template to look old even when the zip had already been refreshed.
