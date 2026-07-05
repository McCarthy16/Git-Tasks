# @tasks/tauri

The desktop app: a [Tauri 2](https://tauri.app) shell with a React + Vite
frontend. This is a client — it renders workspaces and dispatches commands;
all stored data lives behind the [daemon](../server/README.md).

## Running

The app expects the daemon to be reachable, so the usual entry point is the
root script that starts both:

```sh
pnpm dev          # from the repo root: daemon + app
pnpm dev:app      # just the app (daemon must already be running)
pnpm build        # build the desktop app
```

The daemon URL defaults to `http://127.0.0.1:4000` and can be overridden for
development with the `TASKS_DAEMON_URL` env var (read by the Tauri backend).

## Server-driven UI

The frontend never holds routing or domain state. The Rust side
(`src-tauri/src/app`) owns the app state machine and exposes exactly two Tauri
commands:

- **`view`** — called once on boot; returns the full `View` describing what to
  draw.
- **`dispatch(action)`** — called for every interaction; applies the `Action`
  to the app state and returns the freshly rendered `View`.

React (`src/useView.ts`) renders whatever `View` it's given and maps user
input to `Action`s — screens, navigation, and selection are decided in Rust.
(A `view-updated` event covers the one case where state changes without a
dispatch, e.g. a macOS dock-menu click opening a workspace.)

## Layout

```
src/                 React frontend
  useView.ts         the view/dispatch loop
  types.ts           View and Action types mirrored from Rust
  components/        screens (SelectRepo, Projects, Tasks, TaskDetail) + dialogs
src-tauri/src/
  app/               the app state machine: state, actions, view rendering
  daemon.rs          typed HTTP client for the tasks daemon
  lib.rs             Tauri adapter exposing `view` and `dispatch`
```

Data access goes exclusively through `daemon.rs` — one method per daemon
endpoint ([HTTP API](../../docs/http-api.md)), workspace-scoped via the
`?workspace=` query parameter.
