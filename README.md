# pomo-tree 🍅

A menu bar pomodoro timer for macOS and Windows, built with Tauri v2 + Rust.

Named after the restaurant "Pomme no Ki" (ポムの樹). As you complete pomodoros, a fictional tomato tree will grow — bigger tasks bear bigger tomatoes (planned, see Roadmap).

[![CI](https://github.com/TakashiAihara/pomo-tree/actions/workflows/ci.yml/badge.svg)](https://github.com/TakashiAihara/pomo-tree/actions/workflows/ci.yml)

## Features

- Work / short break / long break cycle (default 25/5/15 min, long break every 4 pomodoros)
- Lives in the menu bar — no Dock icon, no window clutter
  - macOS: live countdown next to the clock (`🍅 24:31`, `☕ 05:00`, `⏸` while paused)
  - Windows: state-colored tray icon (red = work / green = break / gray = stopped) with countdown tooltip
- Tray menu: start / pause / resume / skip / reset / settings / quit
- Native notification when a session ends
- Settings window: durations, long-break interval, auto-start next session
- Session history recorded to a local JSONL file (fuel for the future tomato tree)
- Sleep-safe: remaining time is computed from elapsed time, so laptop sleep does not skew the timer

## Install

### macOS (Apple Silicon)

```bash
curl -fsSL https://raw.githubusercontent.com/TakashiAihara/pomo-tree/main/scripts/install.sh | bash
```

Installs the latest release into `/Applications`. Intel Macs are not published yet — build from source (see Development).

### Windows (x64)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/TakashiAihara/pomo-tree/main/scripts/install.ps1 | iex"
```

Runs the latest NSIS installer silently.

### Manual

Grab the `.dmg` / `.msi` / `-setup.exe` from [Releases](https://github.com/TakashiAihara/pomo-tree/releases).

Note: builds are not code-signed. On macOS, a `.dmg` downloaded via a browser is quarantined by Gatekeeper — right-click the app and choose Open the first time, or use the install script above (which clears quarantine).

## Usage

Everything happens from the tray icon:

1. Click the tomato in the menu bar (macOS) or the colored dot in the system tray (Windows)
2. 開始 to start a work session; the countdown shows in the menu bar / tooltip
3. When the session ends you get a notification, and the next phase is queued (or auto-started if enabled in settings)
4. 設定… opens the settings window; closing it hides the window, the timer keeps running

Data locations:

| What | Where |
|------|-------|
| Settings | `settings.json` in the app data dir |
| Session history | `sessions.jsonl` in the app data dir |

App data dir: `~/Library/Application Support/com.takashiaihara.pomo-tree` (macOS), `%APPDATA%/com.takashiaihara.pomo-tree` (Windows).

## Development

Requirements: Rust (stable), Bun, and on macOS the Xcode Command Line Tools.

```bash
bun install
bun run tauri dev     # run with hot reload
bun run tauri build   # produce a release bundle locally
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml     # timer state machine & co.
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```

The app icon is generated from `assets/tomato-icon.svg`:

```bash
rsvg-convert -w 1024 -h 1024 assets/tomato-icon.svg -o /tmp/tomato-1024.png
bun run tauri icon /tmp/tomato-1024.png
```

### Releasing

Push a `v*` tag. GitHub Actions builds macOS / Windows bundles, publishes the release, and attaches fixed-name assets used by the install scripts.

```bash
git tag v0.1.0 && git push origin v0.1.0
```

## Design

Design documents live in `docs/design/`. The v0.1 architecture, platform quirks (why Windows has no menu bar text), and decision log are in `docs/design/v0.1.0-core-timer.md`.

## Roadmap

1. v0.2 — the tomato tree: a tree that grows as you complete pomodoros, rendered from the recorded session history
2. v0.3 — task tree: manage tasks as a tree, attach pomodoros to tasks; bigger tasks bear bigger tomatoes
3. Backlog: login-item autostart, menu bar popover, numeric countdown tray icon on Windows, Intel mac builds
