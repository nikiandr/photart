# photart

A local photo inspector built with Tauri, React, TypeScript, and Vite.
It runs on macOS, Windows, and Linux.

## Photo import

Photart imports JPEG, PNG, WebP, HEIC, and TIFF (`.tif` or `.tiff`). `.heif`
works when it contains a supported HEVC still image. Extensions are case
insensitive and must match the file contents. Unsupported, unreadable, and
oversize files are skipped. Camera RAW, including DNG, is not supported.

Photart copies originals byte-for-byte. Gallery thumbnails are JPEGs up to
640 pixels on the long edge. Detail view uses the original for JPEG, PNG, and
WebP. For HEIC and TIFF it generates and caches a JPEG up to 2,560 pixels
on first opening the photograph in Detail. Those detail images are display
copies, not full-resolution, color-managed, HDR, or
transparent renderings. Capture details are read from the original when
available. See [format limits and decoder choices](docs/photo-import.md).

## Development

Install dependencies and start the Vite app and Storybook together:

```sh
bun install
bun run dev
```

The app is available at `http://localhost:1420` and Storybook at
`http://localhost:6006`.

To launch the Tauri desktop app in development mode (which also starts
Storybook). This build is named Photart Dev, so it can sit beside the
installed Photart app:

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

The installable app is named Photart. On a Mac:

```sh
bun install
bun run package:mac
```

That writes `Photart.app` and a `.dmg` under `src-tauri/target/release/bundle/`.
Drag the app into Applications, or send the `.dmg`. The first build is
ad-hoc signed, so another Mac needs a right-click → Open the first time.

`bun run tauri build` produces the same Photart app for the current platform.

The build compiles the bundled LibRaw source; no system LibRaw or Homebrew
decoder installation is needed. See [LibRaw distribution](docs/libraw-distribution.md)
for the source, notices, and website release checklist.

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

Linux packaging uses `.deb` and `.rpm` (`bun run tauri build --bundles deb,rpm`). Bundles land in `src-tauri/target/release/bundle/deb/` and `src-tauri/target/release/bundle/rpm/`. The `.deb` already depends on WebKitGTK 4.1 at runtime.

A default `bun run tauri build` also tries an AppImage. That needs working FUSE/`linuxdeploy` on the host; build it explicitly with `bun run tauri build --bundles appimage` if you want it.

The GitHub Actions production Build workflow is currently disabled (`build.yml.disabled`). Rename it to `build.yml` to emit those installers in CI.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
