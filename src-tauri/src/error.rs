//! Error types shared across the domain, infrastructure, and app layers.

/// Errors raised while parsing prefixed entity IDs.
#[derive(thiserror::Error, Debug)]
pub enum IdError {
    #[error("id is missing the `{0}` prefix")]
    MissingPrefix(&'static str),
    #[error("id does not contain a valid uuid")]
    InvalidUuid,
}

/// Errors raised by the store, replay logic, and app state machine.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("id error: {0}")]
    Id(#[from] IdError),
    #[error("could not rebuild entity: its event history does not begin with a `created` event")]
    NotCreated,
    #[error("project {0} does not exist")]
    ProjectNotFound(crate::projects::ProjectId),
    #[error("task {0} does not exist")]
    TaskNotFound(crate::tasks::TaskId),
    #[error("status {0} does not exist")]
    StatusNotFound(crate::statuses::StatusId),
    #[error("no workspace is open")]
    NoWorkspace,
    #[error("no project is selected")]
    NoProjectSelected,
}

pub type Result<T> = std::result::Result<T, Error>;
