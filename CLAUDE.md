# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

photart is a Tauri 2 desktop app (macOS & Windows) for amateur photographers who want to make their photography workflow easier and faster. Frontend: React 19 + TypeScript + Vite. Backend: Rust. Package manager/runtime for JS: Bun.

## Commands

```bash
bun install                  # install JS dependencies
bun run tauri dev            # run the full desktop app (compiles Rust, starts Vite)
bun run dev                  # frontend only, in a browser (no Tauri APIs)
bun run build                # typecheck (tsc) + build frontend into dist/
bun run tauri build          # production app bundle
```

Rust checks (match CI, which fails on warnings):

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

- Run `bun run build` before clippy/cargo builds: `tauri::generate_context!` embeds `dist/`, so Rust compilation fails if the frontend hasn't been built.
- The Vite dev server is pinned to port 1420 (`strictPort`) — Tauri needs a fixed dev URL, so only one instance can run at a time.
- There is no test suite yet.

## Architecture

The app is split into two halves that communicate over Tauri's IPC bridge:

- `src/` — React frontend (webview). Calls into Rust with `invoke("command_name", { args })` from `@tauri-apps/api/core`.
- `src-tauri/` — Rust backend. `src/main.rs` is a thin entry point that calls `photart_lib::run()`; all real logic lives in `src/lib.rs`.

To add a backend command: write a `#[tauri::command]` fn in `src-tauri/src/lib.rs` and register it in the `tauri::generate_handler![...]` list in `run()`.

Configuration that matters:

- `src-tauri/tauri.conf.json` — app identity, window config, bundling, and the dev/build hooks (it runs the `bun run dev`/`bun run build` scripts).
- `src-tauri/capabilities/default.json` — permission allowlist for the main window. Tauri plugin APIs are denied by default; when adding a plugin, its permissions must be added here (and the plugin registered in `run()` plus added to both `Cargo.toml` and `package.json`).

## CI (.github/workflows)

- `lint.yml` — every push/PR: `bun run build`, `cargo fmt --check`, `cargo clippy -D warnings`.
- `build.yml` — builds macOS (universal) and Windows bundles as artifacts on every PR/push to main; pushing a `v*` tag creates a draft GitHub Release with installers.
