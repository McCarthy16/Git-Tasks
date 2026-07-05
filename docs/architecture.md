# Architecture

How the system is put together: a core domain crate, a local daemon that owns
it, and thin clients that talk to the daemon over local HTTP.

```
Tauri app (React) ──┐
                    ├──HTTP──▶  tasks-server  ──tasks-core──▶  .tasks/ on disk
tasks CLI ──────────┘          (127.0.0.1:4000)
```

## Workspace layout

The repository is a combined pnpm + Cargo workspace:

| Path | Package | Role |
|---|---|---|
| [`packages/core`](../packages/core/README.md) | `tasks-core` | The domain: events, storage, reconstruction, projections, commands, seeds. |
| [`apps/server`](../apps/server/README.md) | `tasks-server` | The daemon: owns core logic, exposes it as a data API over local HTTP. |
| [`apps/cli`](../apps/cli/README.md) | `tasks-cli` | Terminal client for the daemon (`tasks` binary). |
| [`apps/tauri`](../apps/tauri/README.md) | `@tasks/tauri` | The desktop app: React UI rendering workspaces served by the daemon. |

Each package has its own README covering how to run and work on it.

## The core crate (`tasks-core`)

`tasks-core` is layered so that data flows one way. Events are the source of
truth; everything else is derived from or produces them.

```
commands ──emit──▶ events ──persisted by──▶ storage
                     │
                     ▼
              reconstruction ──produces──▶ projections
```

| Layer | Role |
|---|---|
| `events` | The event definitions — one adjacently-tagged enum per entity, wrapped in a shared envelope. Declares the `EventStore` trait. |
| `storage` | Persists and loads events. Ships the filesystem store (`FsEventStore`), the seed overlay, and an in-memory test double. |
| `reconstruction` | Folds an event stream into current state (`replay`, one per entity). |
| `projections` | The fully reconstructed read-side objects (`Project`, `Task`, `Status`) and their typed IDs. |
| `commands` | The write-side operations: validate intent, append exactly one event, return the rebuilt projection. Also the read helpers (`load`/`list`/`exists`). |

`shared` holds cross-cutting primitives (the typed-ID machinery); `error` holds
the crate's error types.

Two seams keep the layers honest:

- **Everything above storage depends on the `EventStore` trait**, not on the
  filesystem. Commands are generic over `impl EventStore`, so the same logic
  runs against the disk store, the in-memory double, or any future backend.
- **Reconstruction is a blind fold.** It never knows whether a stream includes
  a built-in seed — storage overlays seeds into the stream at read time (see
  [Seeds](seeds.md)), so by the time events reach `replay`, a seed is just an
  ordinary leading snapshot event.

See [Events & Storage](events-data-store.md) for the store itself and
[Data Model](data-model.md) for the entities.

## The daemon (`tasks-server`)

A long-running process that owns access to the stored state and exposes it over
HTTP as plain data operations ([HTTP API](http-api.md)). Built on axum + tokio.

- **Loopback only.** It binds to `127.0.0.1:4000` and nothing else — no network
  exposure, preserving the local-first guarantee.
- **One daemon, every workspace.** The daemon is not tied to a repository. Each
  data request names its workspace with a `?workspace=<repo-root>` query
  parameter, and the handler opens a fresh `FsEventStore` over that root. The
  daemon holds no per-workspace state, so any number of clients can work on any
  number of workspaces concurrently. Concurrent writes are safe by the store's
  design — one immutable file per event.
- **No app logic.** Each route maps a request onto one `tasks_core` operation
  and serializes the result as JSON. Navigation, selection, and screens belong
  to clients.

## Clients

- **Tauri app** (`apps/tauri`) — the desktop UI. React + Vite frontend in a
  Tauri shell; renders workspaces and dispatches commands via the daemon's HTTP
  API.
- **CLI** (`apps/cli`) — a terminal client with the same shape: every command
  maps onto one daemon route, no core logic in the client. The workspace
  defaults to the nearest ancestor directory containing `.tasks` (like git repo
  discovery), overridable with `--workspace`; `--json` emits the daemon's raw
  responses for scripting.
