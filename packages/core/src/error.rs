//! Error types for the core crate.

use crate::projections::{ProjectId, StatusId, TaskId};

/// A `Result` whose error is the crate-wide [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The crate-wide error type.
///
/// Layers convert into this via `?`: the storage layer surfaces filesystem and
/// (de)serialization failures, ID parsing flows in from [`IdError`], and the
/// [`commands`](crate::commands) layer raises the "not found" / "not created"
/// intent failures below when an operation references an entity that doesn't
/// exist or produces a stream that can't be rebuilt.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A parsed ID was malformed.
    #[error(transparent)]
    Id(#[from] IdError),

    /// The underlying store failed a filesystem operation.
    #[error("event store i/o failed: {0}")]
    Io(#[from] std::io::Error),

    /// An event could not be serialized to or deserialized from JSON.
    #[error("event (de)serialization failed: {0}")]
    Serde(#[from] serde_json::Error),

    /// An entity's event history did not fold into a projection — its stream
    /// does not begin with a `created` (or bootstrapping `snapshot`) event.
    #[error("could not rebuild entity: its event history has no bootstrapping event")]
    NotCreated,

    /// A command referenced a project that has no stream in the store.
    #[error("project {0} does not exist")]
    ProjectNotFound(ProjectId),

    /// A command referenced a task that has no stream in the store.
    #[error("task {0} does not exist")]
    TaskNotFound(TaskId),

    /// A command referenced a status that is neither a seed nor on disk (or has
    /// been removed, where an active status was required).
    #[error("status {0} does not exist")]
    StatusNotFound(StatusId),
}

/// Errors raised while parsing prefixed entity IDs.
#[derive(thiserror::Error, Debug)]
pub enum IdError {
    #[error("id is missing the `{0}` prefix")]
    MissingPrefix(&'static str),
    #[error("id does not contain a valid uuid")]
    InvalidUuid,
}
