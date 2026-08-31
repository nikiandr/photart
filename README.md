# photart

A local photo inspector built with Tauri, React, TypeScript, and Vite.
It runs on macOS, Windows, and Linux.

## Development

Install dependencies and start the Vite app and Storybook together:

```sh
bun install
bun run dev
```

The app is available at `http://localhost:1420` and Storybook at
`http://localhost:6006`.

To launch the Tauri desktop app in development mode (which also starts
Storybook):

```sh
bun run tauri dev
```

To start either frontend server independently:

```sh
bun run dev:app
bun run storybook
```

To create a static Storybook build:

```sh
bun run build-storybook
```

## Building

Production installers are produced with Tauri:

```sh
bun install
bun run tauri build
```

On Linux, install the [Tauri v2 system dependencies](https://v2.tauri.app/start/prerequisites/#linux) first, plus Rust (via [rustup](https://rustup.rs/)) and [Bun](https://bun.sh/). On Debian/Ubuntu:

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  xdg-utils \
  rpm
```

CI builds `.deb` and `.rpm` only (`bun run tauri build --bundles deb,rpm`). Bundles land in `src-tauri/target/release/bundle/deb/` and `src-tauri/target/release/bundle/rpm/`. The `.deb` already depends on WebKitGTK 4.1 at runtime.

A default `bun run tauri build` also tries an AppImage. That needs working FUSE/`linuxdeploy` on the host and is not part of CI; build it explicitly with `bun run tauri build --bundles appimage` if you want it.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
