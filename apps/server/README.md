# tasks-server

The tasks daemon: a long-running process that owns access to the stored
state — the event-sourced `.tasks` data inside any workspace — driven through
[`tasks-core`](../../packages/core/README.md) and exposed over local HTTP as
plain data operations.

The daemon is the one process that touches disk. Clients (the desktop app, the
CLI) hold no core logic; they call the daemon. App logic — navigation,
selection, screens — lives in the clients, not here: each route maps a request
onto exactly one core operation and serializes the result as JSON.

The full route reference lives in [docs/http-api.md](../../docs/http-api.md).

## Running

```sh
cargo run -p tasks-server        # listens on http://127.0.0.1:4000
pnpm dev:server                  # same, via the pnpm script
pnpm dev                         # full stack: daemon + desktop app
```

Logging uses `tracing`, honoring `RUST_LOG` (default `info`):

```sh
RUST_LOG=debug cargo run -p tasks-server
```

Shutdown is graceful on Ctrl-C. The listen address is fixed at
`127.0.0.1:4000` — loopback only, by design: nothing is exposed to the
network, preserving the project's offline / local-first guarantee.

## Design notes

- **One daemon, every workspace.** The daemon is not tied to a repository.
  Every data route names its workspace with a `?workspace=<repo-root>` query
  parameter, and each request opens its own `FsEventStore` over that root —
  the daemon holds no per-workspace state, so any number of clients can work
  on any number of workspaces concurrently.
- **Concurrent writes are safe by store design**, not by locking: one
  immutable, uniquely named file per event (see
  [Events & Storage](../../docs/events-data-store.md)).
- **Errors** map onto HTTP statuses in one place (`http/mod.rs`): unknown
  workspace/entity → `404`, malformed ID → `400`, storage failure → `500`,
  always as `{ "error": "…" }`.

## Layout

```
src/
  main.rs        binding, tracing, graceful shutdown
  error.rs       daemon error type, wrapping tasks-core's
  http/          the transport: one module per entity, mirroring
    mod.rs       tasks_core::commands (workspaces, projects, tasks, statuses)
```
