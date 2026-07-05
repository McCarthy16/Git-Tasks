//! Project routes — thin adapters over `tasks_core::commands::project`.
//!
//! Every route is workspace-scoped via the shared [`Ws`] query parameter.

use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use tasks_core::commands::project;
use tasks_core::{Error as CoreError, Project, ProjectId};

use crate::http::{ApiError, Ws};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(load))
        .route("/{id}/rename", post(rename))
        .route("/{id}/close", post(close))
        .route("/{id}/reopen", post(reopen))
}

#[derive(Deserialize, Default)]
struct ListQuery {
    /// List closed (archived) projects instead of open ones.
    #[serde(default)]
    closed: bool,
}

/// List all projects in the workspace, oldest first. `?closed=true` lists
/// archived ones instead.
async fn list(
    Query(ws): Query<Ws>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Project>>, ApiError> {
    let store = ws.store()?;
    let projects = if query.closed {
        project::list_closed(&store)?
    } else {
        project::list(&store)?
    };
    Ok(Json(projects))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
}

/// Create a project, returning it.
async fn create(
    Query(ws): Query<Ws>,
    Json(body): Json<CreateBody>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(project::create(&ws.store()?, body.name)?))
}

/// Load one project, or 404 if it has no events in the store.
async fn load(
    Query(ws): Query<Ws>,
    Path(id): Path<ProjectId>,
) -> Result<Json<Project>, ApiError> {
    let project = project::load(&ws.store()?, id)?.ok_or(CoreError::ProjectNotFound(id))?;
    Ok(Json(project))
}

#[derive(Deserialize)]
struct RenameBody {
    new_name: String,
}

/// Rename a project, returning the rebuilt project.
async fn rename(
    Query(ws): Query<Ws>,
    Path(id): Path<ProjectId>,
    Json(body): Json<RenameBody>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(project::rename(&ws.store()?, id, body.new_name)?))
}

/// Close (archive) a project, returning the rebuilt project.
async fn close(
    Query(ws): Query<Ws>,
    Path(id): Path<ProjectId>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(project::close(&ws.store()?, id)?))
}

/// Reopen (restore) a closed project, returning the rebuilt project.
async fn reopen(
    Query(ws): Query<Ws>,
    Path(id): Path<ProjectId>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(project::reopen(&ws.store()?, id)?))
}
