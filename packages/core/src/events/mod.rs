//! Events — the event code.
//!
//! Definitions of the domain events that are appended to the store and serve as
//! the source of truth for the application. Each entity's events are an
//! adjacently-tagged enum wrapped in the generic [`envelope::Event`] envelope.
//!
//! Three entities are modeled today:
//!
//! - [`project::ProjectEventKind`] — project lifecycle events.
//! - [`task::TaskEventKind`] — task lifecycle events.
//! - [`status::StatusEventKind`] — status lifecycle events.

pub mod envelope;
pub mod project;
pub mod status;
pub mod store;
pub mod task;

pub use envelope::{Event, EventKind};
pub use store::EventStore;

/// Every event-store collection, one per modeled entity. Handy for seeding the
/// full `.tasks` tree in one call (see [`FsEventStore::ensure`]).
///
/// [`FsEventStore::ensure`]: crate::storage::FsEventStore::ensure
pub const COLLECTIONS: &[&str] = &[project::COLLECTION, task::COLLECTION, status::COLLECTION];
pub use project::{ProjectEvent, ProjectEventKind};
pub use status::{StatusEvent, StatusEventKind};
pub use task::{TaskEvent, TaskEventKind};
