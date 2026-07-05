# core

The core of the application: the event-sourced domain logic every client and
the daemon are built on. Events are the source of truth; everything else is
derived from or produces them.

## Layers

| Module           | Role                                                              |
| ---------------- | ----------------------------------------------------------------- |
| `events`         | The event code — definitions of the domain events, the envelope, and the `EventStore` trait. |
| `storage`        | The storage layer — persists and loads events (filesystem store, seed overlay, in-memory test double). |
| `reconstruction` | The process of taking events and folding them into projections.   |
| `projections`    | The fully reconstructed objects (the read-side models) and their typed IDs. |
| `commands`       | The write-side operations that validate intent and emit events, plus the read helpers. |

`shared` holds cross-cutting primitives (typed-ID machinery); `error` holds the
crate's error types.

## How it fits together

```
commands ──emit──▶ events ──persisted by──▶ storage
                     │
                     ▼
              reconstruction ──produces──▶ projections
```

- **Commands** are the write side: they validate intent, append exactly one
  event, and return the rebuilt projection.
- **Events** are appended to and loaded from **storage**, behind the
  `EventStore` trait.
- **Reconstruction** reads events and folds them into **projections**.
- **Projections** are the fully reconstructed objects the rest of the app reads.

The full picture — including seeds and the conflict-free store design — is
documented in [`docs/`](../../docs/README.md), particularly
[Architecture](../../docs/architecture.md),
[Events & Storage](../../docs/events-data-store.md), and
[Seeds](../../docs/seeds.md).

## Workspace

This crate is a member of the root Cargo workspace (`/Cargo.toml`). The package
is named `tasks-core` (imported as `tasks_core`) rather than `core` to avoid
shadowing Rust's built-in `core` crate. The daemon (`apps/server`) and the
Tauri app depend on it via path dependencies.
