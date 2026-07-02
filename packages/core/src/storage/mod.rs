//! Storage — the storage layer.
//!
//! Persists events to and loads events from the underlying store. The layer is
//! defined by the [`EventStore`](crate::events::EventStore) trait (declared
//! alongside the events it stores); everything above depends on that trait, not
//! on any particular backend.
//!
//! - [`fs::FsEventStore`] — the concrete filesystem-backed implementation that
//!   persists events under a workspace's `.tasks` directory.
//! - [`seeds`] — built-in baselines the store serves without anything on disk.
//!   Storage *owns* seeds: it overlays them onto a stream at read time so no
//!   other layer has to know they exist.
//! - `memory::InMemoryEventStore` — a disk-free store double, compiled only
//!   under the `test-util` feature (and core's own tests).

pub mod fs;
pub mod seeds;

pub use fs::FsEventStore;
pub use seeds::{load_stream, stream_ids, Seeded};

#[cfg(any(test, feature = "test-util"))]
pub mod memory;
#[cfg(any(test, feature = "test-util"))]
pub use memory::InMemoryEventStore;
