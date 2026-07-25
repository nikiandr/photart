# photart

A local photo inspector built with Tauri, React, TypeScript, and Vite.

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

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
