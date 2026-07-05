//! Workspace routes.
//!
//! The daemon holds no per-workspace state — a workspace is just a repo root
//! that data routes name via `?workspace=`. The one operation here is
//! initialization: find-or-create the `.tasks` scaffold inside an existing
//! repo so a client can start working in it. Idempotent, so clients may call
//! it on every "open".

use std::path::PathBuf;

use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use tasks_core::events::COLLECTIONS;
use tasks_core::FsEventStore;

use crate::error::Error;
use crate::http::ApiError;

pub fn router() -> Router {
    Router::new().route("/", post(init))
}

#[derive(Deserialize)]
struct InitBody {
    path: PathBuf,
}

/// A workspace described for clients.
#[derive(Serialize)]
struct WorkspaceInfo {
    root: String,
    tasks_dir: String,
}

/// Find-or-create the `.tasks` collection folders inside the repo at `path`.
/// The repo itself must already exist — the daemon initializes workspaces, it
/// doesn't create repositories.
async fn init(Json(body): Json<InitBody>) -> Result<Json<WorkspaceInfo>, ApiError> {
    if !body.path.is_dir() {
        return Err(Error::WorkspaceNotFound(body.path).into());
    }
    let store = FsEventStore::new(&body.path);
    store.ensure(COLLECTIONS)?;
    Ok(Json(WorkspaceInfo {
        root: store.root().display().to_string(),
        tasks_dir: store.dot_tasks().display().to_string(),
    }))
}
