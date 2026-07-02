//! The HTTP transport.
//!
//! A thin shell over the core: it maps requests onto `tasks_core` operations
//! and serializes the results back as JSON. The intent is that all real logic
//! stays in the core crate — this layer only speaks HTTP.
//!
//! The command surface (`view` / `dispatch`) is stubbed until `tasks_core`
//! exposes it; today only `/health` is live so the daemon is runnable and
//! observable end-to-end.

use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

/// Build the daemon's router, wiring every route to the shared [`AppState`].
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

/// Liveness probe. Returns the daemon's status and whether a workspace is open.
async fn health(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<Health> {
    Json(Health {
        status: "ok",
        workspace_open: state.workspace().is_some(),
    })
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    workspace_open: bool,
}

// TODO: once `tasks_core` exposes its command surface, add:
//   GET  /view          -> render current state as a View
//   POST /dispatch      -> apply an Action, return the new View
// Both should be thin adapters that call into the core and (de)serialize JSON.
