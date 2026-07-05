//! The HTTP transport.
//!
//! A thin shell over the core: each route maps a request onto one
//! `tasks_core` operation and serializes the result back as JSON. No app
//! logic lives here — navigation, selection, screens, and anything else
//! UI-shaped belongs to clients.
//!
//! The daemon is not tied to one repository: it serves any number of
//! workspaces to any number of clients at once. Every data route names its
//! workspace with a `?workspace=<repo-root>` query parameter, and each request
//! opens its own [`FsEventStore`] over that root — the daemon holds no
//! per-workspace state, so concurrent clients never contend. (Concurrent
//! writes are safe by the store's design: one immutable file per event.)
//!
//! One module per entity, mirroring `tasks_core::commands`:
//! [`workspaces`], [`projects`], [`tasks`], [`statuses`].

mod projects;
mod statuses;
mod tasks;
mod workspaces;

use std::path::PathBuf;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use tasks_core::FsEventStore;

use crate::error::Error;

/// Build the daemon's router.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/workspaces", workspaces::router())
        .nest("/projects", projects::router())
        .nest("/tasks", tasks::router())
        .nest("/statuses", statuses::router())
}

/// Liveness probe.
async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

/// The `?workspace=<repo-root>` parameter every data route carries.
#[derive(Deserialize)]
pub(crate) struct Ws {
    workspace: PathBuf,
}

impl Ws {
    /// An event store rooted at the named workspace, or
    /// [`Error::WorkspaceNotFound`] if the root doesn't exist on disk.
    pub(crate) fn store(&self) -> Result<FsEventStore, Error> {
        if !self.workspace.is_dir() {
            return Err(Error::WorkspaceNotFound(self.workspace.clone()));
        }
        Ok(FsEventStore::new(&self.workspace))
    }
}

/// An [`Error`] as an HTTP response: a status code and `{ "error": "…" }`.
pub(crate) struct ApiError(Error);

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl From<tasks_core::Error> for ApiError {
    fn from(err: tasks_core::Error) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use tasks_core::Error as Core;
        let status = match &self.0 {
            // The request referenced something that doesn't exist.
            Error::WorkspaceNotFound(_)
            | Error::Core(
                Core::ProjectNotFound(_) | Core::TaskNotFound(_) | Core::StatusNotFound(_),
            ) => StatusCode::NOT_FOUND,
            // Malformed IDs arrive inside request payloads.
            Error::Core(Core::Id(_)) => StatusCode::BAD_REQUEST,
            // Storage / rebuild failures are the daemon's fault.
            Error::Core(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use tower::util::ServiceExt;

    use super::*;

    /// Drive one request through the router, returning (status, parsed body).
    async fn call(router: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(match body {
                Some(v) => Body::from(v.to_string()),
                None => Body::empty(),
            })
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }

    #[tokio::test]
    async fn health_is_live() {
        let (status, body) = call(&router(), "GET", "/health", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], json!("ok"));
    }

    #[tokio::test]
    async fn init_scaffolds_a_workspace() {
        let router = router();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().display().to_string();

        let (status, body) =
            call(&router, "POST", "/workspaces", Some(json!({ "path": path }))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["root"], json!(path));
        assert!(tmp.path().join(".tasks/projects").is_dir());
    }

    #[tokio::test]
    async fn unknown_workspace_root_is_not_found() {
        let router = router();
        let (status, body) =
            call(&router, "GET", "/projects?workspace=/definitely/not/a/repo", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body["error"].as_str().unwrap().contains("workspace root"));
    }

    #[tokio::test]
    async fn full_project_and_task_flow() {
        let router = router();
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().display().to_string();

        // Fetch all projects (empty), create one, fetch again.
        let (status, listed) = call(&router, "GET", &format!("/projects?workspace={ws}"), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(listed, json!([]));

        let (status, project) = call(
            &router,
            "POST",
            &format!("/projects?workspace={ws}"),
            Some(json!({ "name": "Roadmap" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let project_id = project["id"].as_str().unwrap().to_string();

        let (_, listed) = call(&router, "GET", &format!("/projects?workspace={ws}"), None).await;
        assert_eq!(listed.as_array().unwrap().len(), 1);

        // Create two tasks in the project and fetch all tasks in it.
        for name in ["First", "Second"] {
            let (status, _) = call(
                &router,
                "POST",
                &format!("/tasks?workspace={ws}"),
                Some(json!({ "project_id": project_id, "name": name })),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
        }
        let (status, tasks) = call(
            &router,
            "GET",
            &format!("/tasks?workspace={ws}&project_id={project_id}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let tasks = tasks.as_array().unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0]["name"], json!("First"));

        // Rename one and read its events.
        let task_id = tasks[0]["id"].as_str().unwrap();
        let (status, task) = call(
            &router,
            "POST",
            &format!("/tasks/{task_id}/rename?workspace={ws}"),
            Some(json!({ "new_name": "First!" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(task["name"], json!("First!"));

        let (_, events) = call(
            &router,
            "GET",
            &format!("/tasks/{task_id}/events?workspace={ws}"),
            None,
        )
        .await;
        let events = events.as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1]["type"], json!("renamed"));
        assert!(events[0]["created_at_millis"].is_u64());

        // Statuses are served from seeds with nothing on disk.
        let (_, statuses) = call(&router, "GET", &format!("/statuses?workspace={ws}"), None).await;
        assert!(!statuses.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn serves_many_workspaces_independently() {
        let router = router();
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let (wa, wb) = (a.path().display().to_string(), b.path().display().to_string());

        // Interleaved writes against two repos through the same daemon.
        call(&router, "POST", &format!("/projects?workspace={wa}"), Some(json!({ "name": "A1" }))).await;
        call(&router, "POST", &format!("/projects?workspace={wb}"), Some(json!({ "name": "B1" }))).await;
        call(&router, "POST", &format!("/projects?workspace={wa}"), Some(json!({ "name": "A2" }))).await;

        let (_, in_a) = call(&router, "GET", &format!("/projects?workspace={wa}"), None).await;
        let (_, in_b) = call(&router, "GET", &format!("/projects?workspace={wb}"), None).await;
        let names = |v: &Value| {
            v.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap().to_string()).collect::<Vec<_>>()
        };
        assert_eq!(names(&in_a), vec!["A1", "A2"]);
        assert_eq!(names(&in_b), vec!["B1"]);
    }

    #[tokio::test]
    async fn missing_entities_map_to_not_found() {
        let router = router();
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().display().to_string();

        let missing = tasks_core::ProjectId::new();
        let (status, _) =
            call(&router, "GET", &format!("/projects/{missing}?workspace={ws}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = call(
            &router,
            "POST",
            &format!("/projects/{missing}/rename?workspace={ws}"),
            Some(json!({ "new_name": "x" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
