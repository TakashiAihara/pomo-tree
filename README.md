# pomo-tree

A menu bar pomodoro timer for macOS (and Windows system tray), built with Tauri v2 + Rust.

Named after the restaurant "Pomme no Ki" (ポムの樹). As you complete pomodoros, a fictional tomato tree will grow — bigger tasks bear bigger tomatoes (planned for v0.2+).

## Features (v0.1)

- Work / short break / long break cycle (default 25/5/15 min)
- Live countdown in the macOS menu bar (tooltip + colored icon on Windows)
- Tray menu: start / pause / resume / skip / reset
- Native notifications on session end
- Settings window

## Development

```bash
bun install
bun run tauri dev
```

## Design

See `docs/design/` for design documents.
