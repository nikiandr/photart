# AGENTS.md

## Cursor Cloud specific instructions

photart is a **Tauri 2** desktop app (React 19 + Vite frontend, Rust backend). JS tooling uses **Bun**. See `CLAUDE.md` and `README.md` for standard commands.

### Services

| Service | Command | URL / notes |
|--------|---------|-------------|
| Full desktop app | `bun run tauri dev` | Native window; also starts Vite + Storybook via `beforeDevCommand` |
| Frontend only | `bun run dev:app` | http://localhost:1420 (port is **strict** — only one instance) |
| Storybook | `bun run storybook` | http://localhost:6006 |
| App + Storybook | `bun run dev` | Both of the above concurrently |

On Linux cloud VMs, Tauri needs a working X display (`DISPLAY` is usually `:1`). You may see harmless `libEGL` / DRI3 warnings in headless environments; the window can still render.

### Lint / build (matches CI)

Run from repo root:

```bash
bun run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

`bun run build` must run **before** `cargo clippy` because `tauri::generate_context!` embeds `dist/`.

There is no frontend test suite. Rust unit tests live under `src-tauri/`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Linux-only gotchas

- **System packages** for Tauri on Ubuntu are listed in `.github/workflows/lint.yml` (`libwebkit2gtk-4.1-dev`, etc.). They are not installed by `bun install`.
- **Rust toolchain**: use a recent `stable` via rustup (currently needs Cargo ≥ 1.85 for some transitive crates). CI uses `dtolnay/rust-toolchain@stable`.
- **Bun**: not preinstalled on all base images; install with the official script if `bun` is missing, then ensure `~/.bun/bin` is on `PATH`.

### Smoke-testing the environment

Basic verification is enough: confirm `bun run build` succeeds, optionally run the Rust checks above, and that `bun run tauri dev` (or `bun run dev:app`) starts and serves the UI. No need to exercise photo upload/analysis flows during environment setup.

## Frontend design

Before any frontend or UI work (components, screens, styling, typography, motion, UI copy), read and follow `.claude/skills/photart-design/SKILL.md`. It is the binding design guideline for this project: atmospheres, tokens, typography ladder, motion budget, and voice. Reference token values by name from `src/styles/globals.css`; rationale lives in `docs/design/design-philosophy.md`.
