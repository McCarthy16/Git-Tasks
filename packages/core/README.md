# core

The core of the application. This crate is where the application logic that
currently lives in the `src-tauri` Tauri app will be consolidated and better
organized.

> **Status:** skeleton / planning. Nothing has been moved yet — this crate lays
> out the intended structure so logic can be migrated out of `src-tauri`
> incrementally.

## Layers

The crate is organized around an event-sourced flow. Events are the source of
truth; everything else is derived from or produces them.

| Module           | Role                                                              |
| ---------------- | ----------------------------------------------------------------- |
| `events`         | The event code — definitions of the domain events.                |
| `storage`        | The storage layer — persists and loads events.                    |
| `reconstruction` | The process of taking events and folding them into projections.   |
| `projections`    | The fully reconstructed objects (the read-side models).           |
| `commands`       | The write-side operations that validate intent and emit events.   |

## How it fits together

```
commands ──emit──▶ events ──persisted by──▶ storage
                     │
                     ▼
              reconstruction ──produces──▶ projections
```

- **Commands** are the write side: they validate intent and produce new events.
- **Events** are appended to and loaded from **storage**.
- **Reconstruction** reads events and folds them into **projections**.
- **Projections** are the fully reconstructed objects the rest of the app reads.

## Workspace

This crate is a member of the root Cargo workspace (`/Cargo.toml`). The package
is named `tasks-core` (imported as `tasks_core`) rather than `core` to avoid
shadowing Rust's built-in `core` crate. The Tauri app (`apps/tauri/src-tauri`)
depends on it via a path dependency.
