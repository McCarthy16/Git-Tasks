//! Task routes — thin adapters over `tasks_core::commands::task`.
//!
//! Every route is workspace-scoped via the shared [`Ws`] query parameter.
//! The `rename` and `description` writes take an optional `event_id`: when
//! present, the write overwrites that event in place (session-scoped
//! deduplication) instead of appending a new one.

use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use tasks_core::commands::task;
use tasks_core::events::task::TaskEventKind;
use tasks_core::{Error as CoreError, EventId, ProjectId, StatusId, Task, TaskId};

use crate::http::{ApiError, Ws};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(load))
        .route("/{id}/events", get(events))
        .route("/{id}/rename", post(rename))
        .route("/{id}/move", post(move_to_project))
        .route("/{id}/close", post(close))
        .route("/{id}/reopen", post(reopen))
        .route("/{id}/status", post(set_status))
        .route("/{id}/description", post(update_description))
}

#[derive(Deserialize, Default)]
struct ListQuery {
    /// Restrict the list to one project.
    project_id: Option<ProjectId>,
    /// List closed tasks instead of open ones.
    #[serde(default)]
    closed: bool,
}

/// List tasks in the workspace, oldest first. `?project_id=…` scopes to all
/// tasks in one project; `?closed=true` lists closed tasks instead of open ones.
async fn list(
    Query(ws): Query<Ws>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let store = ws.store()?;
    let tasks = match (query.project_id, query.closed) {
        (Some(pid), false) => task::list_in_project(&store, pid)?,
        (Some(pid), true) => task::list_closed_in_project(&store, pid)?,
        (None, false) => task::list(&store)?,
        (None, true) => task::list_closed(&store)?,
    };
    Ok(Json(tasks))
}

#[derive(Deserialize)]
struct CreateBody {
    project_id: ProjectId,
    name: String,
}

/// Create a task within an existing project, returning it.
async fn create(
    Query(ws): Query<Ws>,
    Json(body): Json<CreateBody>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(task::create(&ws.store()?, body.project_id, body.name)?))
}

/// Load one task, or 404 if it has no events in the store.
async fn load(Query(ws): Query<Ws>, Path(id): Path<TaskId>) -> Result<Json<Task>, ApiError> {
    let task = task::load(&ws.store()?, id)?.ok_or(CoreError::TaskNotFound(id))?;
    Ok(Json(task))
}

/// A task event enriched with its decoded creation time, so clients don't
/// have to decode UUIDv7 timestamps themselves. Serializes as
/// `{ "id": …, "created_at_millis": …, "type": …, "payload": … }`.
#[derive(Serialize)]
struct TaskEventDto {
    id: String,
    created_at_millis: Option<u64>,
    #[serde(flatten)]
    kind: TaskEventKind,
}

/// The raw event history of a task, oldest first.
async fn events(
    Query(ws): Query<Ws>,
    Path(id): Path<TaskId>,
) -> Result<Json<Vec<TaskEventDto>>, ApiError> {
    let store = ws.store()?;
    task::load(&store, id)?.ok_or(CoreError::TaskNotFound(id))?;
    let events = task::load_events(&store, id)?
        .into_iter()
        .map(|e| TaskEventDto {
            id: e.id.to_string(),
            created_at_millis: e.created_at_millis(),
            kind: e.kind,
        })
        .collect();
    Ok(Json(events))
}

#[derive(Deserialize)]
struct RenameBody {
    new_name: String,
    /// When set, overwrite this event in place instead of appending.
    event_id: Option<EventId>,
}

/// Rename a task (in place when `event_id` is given), returning the rebuilt task.
async fn rename(
    Query(ws): Query<Ws>,
    Path(id): Path<TaskId>,
    Json(body): Json<RenameBody>,
) -> Result<Json<Task>, ApiError> {
    let store = ws.store()?;
    let task = match body.event_id {
        Some(event_id) => task::rename_in_place(&store, id, event_id, body.new_name)?,
        None => task::rename(&store, id, body.new_name)?,
    };
    Ok(Json(task))
}

#[derive(Deserialize)]
struct MoveBody {
    project_id: ProjectId,
}

/// Move a task to a different project, returning the rebuilt task.
async fn move_to_project(
    Query(ws): Query<Ws>,
    Path(id): Path<TaskId>,
    Json(body): Json<MoveBody>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(task::move_to_project(&ws.store()?, id, body.project_id)?))
}

/// Close a task, returning the rebuilt task.
async fn close(Query(ws): Query<Ws>, Path(id): Path<TaskId>) -> Result<Json<Task>, ApiError> {
    Ok(Json(task::close(&ws.store()?, id)?))
}

/// Reopen a closed task, returning the rebuilt task.
async fn reopen(Query(ws): Query<Ws>, Path(id): Path<TaskId>) -> Result<Json<Task>, ApiError> {
    Ok(Json(task::reopen(&ws.store()?, id)?))
}

#[derive(Deserialize)]
struct SetStatusBody {
    /// `null` clears the status.
    status_id: Option<StatusId>,
}

/// Set (or clear) a task's status, returning the rebuilt task.
async fn set_status(
    Query(ws): Query<Ws>,
    Path(id): Path<TaskId>,
    Json(body): Json<SetStatusBody>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(task::set_status(&ws.store()?, id, body.status_id)?))
}

#[derive(Deserialize)]
struct DescriptionBody {
    description: String,
    /// When set, overwrite this event in place instead of appending.
    event_id: Option<EventId>,
}

/// Update a task's description (in place when `event_id` is given), returning
/// the rebuilt task.
async fn update_description(
    Query(ws): Query<Ws>,
    Path(id): Path<TaskId>,
    Json(body): Json<DescriptionBody>,
) -> Result<Json<Task>, ApiError> {
    let store = ws.store()?;
    let task = match body.event_id {
        Some(event_id) => {
            task::update_description_in_place(&store, id, event_id, body.description)?
        }
        None => task::update_description(&store, id, body.description)?,
    };
    Ok(Json(task))
}
