//! Core application logic.
//!
//! This crate bundles the domain logic that currently lives in the `src-tauri`
//! app. Nothing is moved yet — this is a skeleton to plan a cleaner
//! organization. The layers:
//!
//! - [`events`]: the event definitions (the source of truth).
//! - [`storage`]: the storage layer that persists and loads events.
//! - [`reconstruction`]: the process of folding events into projections.
//! - [`projections`]: the fully reconstructed, read-side objects.
//! - [`commands`]: the write-side operations that produce new events.
//!
//! [`shared`] holds cross-cutting foundation primitives (typed IDs) the layers
//! are defined in terms of; [`error`] holds the crate's error types.

pub mod commands;
pub mod error;
pub mod events;
pub mod projections;
pub mod reconstruction;
pub mod shared;
pub mod storage;

// --- Crate-root re-exports -------------------------------------------------
//
// The handful of types a consumer (e.g. the daemon/server) reaches for most,
// surfaced at the crate root so callers can `use tasks_core::{...}` without
// threading full module paths. The write commands stay namespaced under
// [`commands`] (`commands::project::create`, …) — flattening them would collide
// three `create`s into one name.

pub use error::{Error, Result};
pub use events::EventStore;
pub use projections::{Project, ProjectId, Status, StatusId, StatusKind, Task, TaskId};
pub use shared::id::EventId;
pub use storage::FsEventStore;
