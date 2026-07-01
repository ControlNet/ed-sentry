# ed-sentry

**ed-sentry** is an Elite Dangerous AFK sentry. It reads your local Journal files, watches combat/session signals, and shows the current state in a tactical desktop/Web dashboard.

It is unofficial third-party software. It only reads local Journal, `Status.json`, and `Cargo.json` files; it does not inject into the game, press keys, relog, or automate the client.

## What it does

- Watches the newest Elite Dangerous `Journal.*.log` file.
- Tracks scans, kills, bounties, massacre mission progress, ship/fighter damage, cargo loss, fuel, death, and session summaries.
- Shows an AFK checklist for hardpoints, engine pips, and cargo state.
- Provides a local WebUI / desktop dashboard.
- Can send watch-mode alerts to an unencrypted Matrix room.
- Can start a Cloudflare Quick Tunnel from the dashboard for temporary remote access.
- Can replay a selected Journal file from the terminal.

## Download

Download the latest package from GitHub Releases:

```text
https://github.com/ControlNet/ed-sentry/releases
```

Release packages are intended to be built by GitHub CI when a version tag is published. Pick the package for your platform, unzip it, and edit the included `config.toml` before enabling Matrix, WebUI, or tunnel passwords.

## Which program do I run?

On Windows desktop packages, run:

```text
ed-sentry.exe
```

Keep these files together in the same folder:

```text
ed-sentry.exe
ed-sentry-core.exe
config.toml
webui/
WebView2Loader.dll
```

`ed-sentry.exe` is the desktop launcher. It starts `ed-sentry-core.exe --gui` for you.

For terminal-only use, run the core binary directly:

```powershell
.\ed-sentry-core.exe --config config.toml
```

Linux packages provide the terminal binary:

```bash
./ed-sentry --config config.toml
```

## Quick start

1. Download and unzip a release package.
2. Open `config.toml` in a text editor.
3. Set your Journal folder if the default Windows Saved Games path is not enough.
4. Set `[web] enabled = true` if you want the dashboard.
5. Run `ed-sentry.exe` on Windows, or `./ed-sentry --config config.toml` on Linux.

On Windows, leaving the Journal folder empty uses the normal Elite Dangerous Saved Games location:

```text
<Saved Games>\Frontier Developments\Elite Dangerous
```

## Minimal config

```toml
[journal]
# Empty on Windows means: <Saved Games>\Frontier Developments\Elite Dangerous
# Set an explicit folder if your Journals are elsewhere.
folder = ""

[web]
enabled = true
host = "127.0.0.1"
port = 8765

[matrix]
enabled = false
homeserver = "https://matrix.example.org"
room_id = "#alerts:example.org"
access_token = "<token>"
mention_user_id = ""

[tunnel]
provider = "cloudflare_quick"
auto_start = false
config_password = ""
```

Important notes:

- Do not share your real `config.toml` if it contains a Matrix access token or tunnel password.
- Matrix delivery requires an unencrypted Matrix room.
- A non-empty tunnel `config_password` requires remote tunnel visitors to log in before using config APIs.
- Local dashboard access does not require the tunnel password.

## Common CLI commands

Watch with a config file:

```bash
ed-sentry-core --config config.toml
```

Watch a specific Journal folder:

```bash
ed-sentry-core --journal "C:\Users\you\Saved Games\Frontier Developments\Elite Dangerous"
```

Replay one Journal file in the terminal:

```bash
ed-sentry-core --replay --set-file "Journal.250101000000.01.log" --no-status-line
```

Useful flags:

- `--config <file>`: load a TOML config file.
- `--journal <folder>`: set the Journal folder.
- `--set-file <file>`: select one Journal file.
- `--replay`: replay a file and exit.
- `--debug`: print startup diagnostics.
- `--no-status-line`: disable the live terminal status line.

## Building locally

Most users should use Releases. If you are building a local Windows GNU package from source:

```bash
./scripts/package-windows-gnu.sh
```

The output is:

```text
dist/ed-sentry-x86_64-pc-windows-gnu.zip
```
