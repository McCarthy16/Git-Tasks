//! Error types for the daemon.

use std::path::PathBuf;

/// The daemon-wide error type: the one precondition the daemon itself
/// enforces, plus everything that bubbles up from the core.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A request referenced a workspace root that doesn't exist on disk.
    #[error("workspace root does not exist: {}", .0.display())]
    WorkspaceNotFound(PathBuf),

    /// A domain or storage failure from the core crate.
    #[error(transparent)]
    Core(#[from] tasks_core::Error),
}
