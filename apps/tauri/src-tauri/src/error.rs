//! Error types for the app layer.

/// Errors raised by the app state machine and the daemon client.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no workspace is open")]
    NoWorkspace,
    #[error("no project is selected")]
    NoProjectSelected,
    /// The daemon answered with an error status; `message` is its own
    /// `{"error": …}` payload.
    #[error("{message}")]
    Daemon { status: u16, message: String },
    /// The daemon could not be reached at all — most likely it isn't running.
    #[error("could not reach the tasks daemon: {0}")]
    DaemonUnreachable(String),
}

pub type Result<T> = std::result::Result<T, Error>;
