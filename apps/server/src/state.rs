//! The daemon's shared runtime state.
//!
//! [`AppState`] is the single value handed to every request handler. It holds
//! whatever the daemon needs to live longer than one request — today just the
//! active workspace (the repo root whose `.tasks/` directory backs the event
//! store). It is cheap to clone (`Arc` inside) so axum can share it across
//! handlers and tasks.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Shared, cloneable handle to the daemon's runtime state.
#[derive(Clone, Default)]
pub struct AppState {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    /// The currently opened workspace (repo root), if any. `None` until a
    /// client opens one. Guarded so handlers can swap workspaces at runtime.
    workspace: RwLock<Option<PathBuf>>,
}

impl AppState {
    /// Create an empty state with no workspace opened yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The currently opened workspace, if any.
    pub fn workspace(&self) -> Option<PathBuf> {
        self.inner
            .workspace
            .read()
            .expect("workspace lock poisoned")
            .clone()
    }

    /// Open (or switch to) a workspace at `root`.
    // Wired in once the command surface (which opens workspaces) lands.
    #[allow(dead_code)]
    pub fn set_workspace(&self, root: PathBuf) {
        *self
            .inner
            .workspace
            .write()
            .expect("workspace lock poisoned") = Some(root);
    }
}
