//! Status routes — thin adapters over `tasks_core::commands::status`.
//!
//! Every route is workspace-scoped via the shared [`Ws`] query parameter.
//! Statuses ship with built-in seeds the store overlays at read time, so the
//! list is never empty even on a fresh workspace; seeds are edited exactly
//! like user-created statuses.

use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use tasks_core::commands::status;
use tasks_core::{Status, StatusId, StatusKind};

use crate::http::{ApiError, Ws};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}/rename", post(rename))
        .route("/{id}/description", post(update_description))
        .route("/{id}/kind", post(change_kind))
        .route("/{id}/remove", post(remove))
}

#[derive(Deserialize, Default)]
struct ListQuery {
    /// List removed statuses instead of active ones.
    #[serde(default)]
    removed: bool,
}

/// List statuses in canonical (seeds-first) order. `?removed=true` lists
/// soft-removed ones instead.
async fn list(
    Query(ws): Query<Ws>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Status>>, ApiError> {
    let store = ws.store()?;
    let statuses = if query.removed {
        status::list_removed(&store)?
    } else {
        status::list(&store)?
    };
    Ok(Json(statuses))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    kind: StatusKind,
    description: Option<String>,
}

/// Create a user-defined status, returning it.
async fn create(
    Query(ws): Query<Ws>,
    Json(body): Json<CreateBody>,
) -> Result<Json<Status>, ApiError> {
    Ok(Json(status::create(
        &ws.store()?,
        body.name,
        body.kind,
        body.description,
    )?))
}

#[derive(Deserialize)]
struct RenameBody {
    new_name: String,
}

/// Rename a status, returning the rebuilt status.
async fn rename(
    Query(ws): Query<Ws>,
    Path(id): Path<StatusId>,
    Json(body): Json<RenameBody>,
) -> Result<Json<Status>, ApiError> {
    Ok(Json(status::rename(&ws.store()?, id, body.new_name)?))
}

#[derive(Deserialize)]
struct DescriptionBody {
    /// `null` clears the description.
    description: Option<String>,
}

/// Update a status's description (`null` clears it), returning the rebuilt status.
async fn update_description(
    Query(ws): Query<Ws>,
    Path(id): Path<StatusId>,
    Json(body): Json<DescriptionBody>,
) -> Result<Json<Status>, ApiError> {
    Ok(Json(status::update_description(&ws.store()?, id, body.description)?))
}

#[derive(Deserialize)]
struct KindBody {
    new_kind: StatusKind,
}

/// Change the semantic kind of a status, returning the rebuilt status.
async fn change_kind(
    Query(ws): Query<Ws>,
    Path(id): Path<StatusId>,
    Json(body): Json<KindBody>,
) -> Result<Json<Status>, ApiError> {
    Ok(Json(status::change_kind(&ws.store()?, id, body.new_kind)?))
}

/// Soft-remove a status, returning the rebuilt status.
async fn remove(
    Query(ws): Query<Ws>,
    Path(id): Path<StatusId>,
) -> Result<Json<Status>, ApiError> {
    Ok(Json(status::remove(&ws.store()?, id)?))
}
