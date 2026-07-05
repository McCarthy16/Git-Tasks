//! The daemon client: all data access goes through the tasks daemon.
//!
//! The Tauri server owns app logic only (navigation, selection, views); the
//! stored information lives behind the daemon (`apps/server`), which serves
//! every workspace over local HTTP. This module is the typed client for that
//! API — one method per endpoint, workspace-scoped via the `?workspace=`
//! query parameter, blocking because the app state machine is synchronous.

use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;

use tasks_core::events::task::TaskEventKind;
use tasks_core::{EventId, Project, ProjectId, Status, StatusId, StatusKind, Task, TaskId};

use crate::error::{Error, Result};

/// Where the daemon listens. Overridable for development via `TASKS_DAEMON_URL`.
const DEFAULT_BASE_URL: &str = "http://127.0.0.1:4000";

/// A typed client for the tasks daemon.
pub struct Daemon {
    base: String,
    agent: ureq::Agent,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            base: std::env::var("TASKS_DAEMON_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string()),
            agent: ureq::Agent::new(),
        }
    }
}

/// A task event as served by the daemon: the raw event enriched with its
/// decoded creation time.
#[derive(Clone, Debug, Deserialize)]
pub struct TaskEvent {
    pub id: String,
    pub created_at_millis: Option<u64>,
    #[serde(flatten)]
    pub kind: TaskEventKind,
}

impl Daemon {
    // --- Workspaces -------------------------------------------------------

    /// Find-or-create the `.tasks` scaffold in the repo at `root`. Idempotent;
    /// called on every workspace open.
    pub fn init_workspace(&self, root: &Path) -> Result<()> {
        let _: serde_json::Value =
            self.post("/workspaces", &[], json!({ "path": root }))?;
        Ok(())
    }

    // --- Projects ---------------------------------------------------------

    /// All open projects in the workspace, oldest first.
    pub fn list_projects(&self, ws: &Path) -> Result<Vec<Project>> {
        self.get("/projects", &[("workspace", &ws_str(ws))])
    }

    /// One project, or `None` if it doesn't exist.
    pub fn load_project(&self, ws: &Path, id: ProjectId) -> Result<Option<Project>> {
        self.get_opt(&format!("/projects/{id}"), &[("workspace", &ws_str(ws))])
    }

    pub fn create_project(&self, ws: &Path, name: String) -> Result<Project> {
        self.post("/projects", &[("workspace", &ws_str(ws))], json!({ "name": name }))
    }

    pub fn rename_project(&self, ws: &Path, id: ProjectId, new_name: String) -> Result<Project> {
        self.post(
            &format!("/projects/{id}/rename"),
            &[("workspace", &ws_str(ws))],
            json!({ "new_name": new_name }),
        )
    }

    pub fn close_project(&self, ws: &Path, id: ProjectId) -> Result<Project> {
        self.post(&format!("/projects/{id}/close"), &[("workspace", &ws_str(ws))], json!({}))
    }

    pub fn reopen_project(&self, ws: &Path, id: ProjectId) -> Result<Project> {
        self.post(&format!("/projects/{id}/reopen"), &[("workspace", &ws_str(ws))], json!({}))
    }

    // --- Tasks ------------------------------------------------------------

    /// All open tasks in one project, oldest first.
    pub fn list_tasks_in_project(&self, ws: &Path, project_id: ProjectId) -> Result<Vec<Task>> {
        self.get(
            "/tasks",
            &[("workspace", &ws_str(ws)), ("project_id", &project_id.to_string())],
        )
    }

    /// One task, or `None` if it doesn't exist.
    pub fn load_task(&self, ws: &Path, id: TaskId) -> Result<Option<Task>> {
        self.get_opt(&format!("/tasks/{id}"), &[("workspace", &ws_str(ws))])
    }

    /// The raw event history of a task, oldest first.
    pub fn task_events(&self, ws: &Path, id: TaskId) -> Result<Vec<TaskEvent>> {
        self.get(&format!("/tasks/{id}/events"), &[("workspace", &ws_str(ws))])
    }

    pub fn create_task(&self, ws: &Path, project_id: ProjectId, name: String) -> Result<Task> {
        self.post(
            "/tasks",
            &[("workspace", &ws_str(ws))],
            json!({ "project_id": project_id, "name": name }),
        )
    }

    /// Rename a task; in place (overwriting `event_id`) when one is given.
    pub fn rename_task(
        &self,
        ws: &Path,
        id: TaskId,
        new_name: String,
        event_id: Option<EventId>,
    ) -> Result<Task> {
        self.post(
            &format!("/tasks/{id}/rename"),
            &[("workspace", &ws_str(ws))],
            json!({ "new_name": new_name, "event_id": event_id }),
        )
    }

    pub fn move_task(&self, ws: &Path, id: TaskId, project_id: ProjectId) -> Result<Task> {
        self.post(
            &format!("/tasks/{id}/move"),
            &[("workspace", &ws_str(ws))],
            json!({ "project_id": project_id }),
        )
    }

    pub fn close_task(&self, ws: &Path, id: TaskId) -> Result<Task> {
        self.post(&format!("/tasks/{id}/close"), &[("workspace", &ws_str(ws))], json!({}))
    }

    pub fn reopen_task(&self, ws: &Path, id: TaskId) -> Result<Task> {
        self.post(&format!("/tasks/{id}/reopen"), &[("workspace", &ws_str(ws))], json!({}))
    }

    /// Set (or clear, with `None`) a task's status.
    pub fn set_task_status(
        &self,
        ws: &Path,
        id: TaskId,
        status_id: Option<StatusId>,
    ) -> Result<Task> {
        self.post(
            &format!("/tasks/{id}/status"),
            &[("workspace", &ws_str(ws))],
            json!({ "status_id": status_id }),
        )
    }

    /// Update a task's description; in place (overwriting `event_id`) when one
    /// is given.
    pub fn update_task_description(
        &self,
        ws: &Path,
        id: TaskId,
        description: String,
        event_id: Option<EventId>,
    ) -> Result<Task> {
        self.post(
            &format!("/tasks/{id}/description"),
            &[("workspace", &ws_str(ws))],
            json!({ "description": description, "event_id": event_id }),
        )
    }

    // --- Statuses ---------------------------------------------------------

    /// All active statuses, seeds first in canonical order.
    pub fn list_statuses(&self, ws: &Path) -> Result<Vec<Status>> {
        self.get("/statuses", &[("workspace", &ws_str(ws))])
    }

    pub fn create_status(
        &self,
        ws: &Path,
        name: String,
        kind: StatusKind,
        description: Option<String>,
    ) -> Result<Status> {
        self.post(
            "/statuses",
            &[("workspace", &ws_str(ws))],
            json!({ "name": name, "kind": kind, "description": description }),
        )
    }

    pub fn rename_status(&self, ws: &Path, id: StatusId, new_name: String) -> Result<Status> {
        self.post(
            &format!("/statuses/{id}/rename"),
            &[("workspace", &ws_str(ws))],
            json!({ "new_name": new_name }),
        )
    }

    /// Update a status's description; `None` clears it.
    pub fn update_status_description(
        &self,
        ws: &Path,
        id: StatusId,
        description: Option<String>,
    ) -> Result<Status> {
        self.post(
            &format!("/statuses/{id}/description"),
            &[("workspace", &ws_str(ws))],
            json!({ "description": description }),
        )
    }

    pub fn change_status_kind(&self, ws: &Path, id: StatusId, new_kind: StatusKind) -> Result<Status> {
        self.post(
            &format!("/statuses/{id}/kind"),
            &[("workspace", &ws_str(ws))],
            json!({ "new_kind": new_kind }),
        )
    }

    pub fn remove_status(&self, ws: &Path, id: StatusId) -> Result<Status> {
        self.post(&format!("/statuses/{id}/remove"), &[("workspace", &ws_str(ws))], json!({}))
    }

    // --- Plumbing ---------------------------------------------------------

    fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        let mut request = self.agent.get(&format!("{}{}", self.base, path));
        for (key, value) in query {
            request = request.query(key, value);
        }
        parse(request.call())
    }

    /// Like [`get`](Self::get), but maps a 404 to `None`.
    fn get_opt<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<Option<T>> {
        let mut request = self.agent.get(&format!("{}{}", self.base, path));
        for (key, value) in query {
            request = request.query(key, value);
        }
        match request.call() {
            Err(ureq::Error::Status(404, _)) => Ok(None),
            result => parse(result).map(Some),
        }
    }

    fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        body: serde_json::Value,
    ) -> Result<T> {
        let mut request = self.agent.post(&format!("{}{}", self.base, path));
        for (key, value) in query {
            request = request.query(key, value);
        }
        parse(request.send_json(body))
    }
}

/// Decode a daemon response, mapping error statuses to [`Error::Daemon`] (with
/// the daemon's own `{"error": …}` message) and transport failures to
/// [`Error::DaemonUnreachable`].
fn parse<T: DeserializeOwned>(result: std::result::Result<ureq::Response, ureq::Error>) -> Result<T> {
    match result {
        Ok(response) => response
            .into_json()
            .map_err(|e| Error::Daemon { status: 200, message: format!("bad response body: {e}") }),
        Err(ureq::Error::Status(status, response)) => {
            let message = response
                .into_json::<serde_json::Value>()
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_else(|| format!("daemon returned status {status}"));
            Err(Error::Daemon { status, message })
        }
        Err(ureq::Error::Transport(t)) => Err(Error::DaemonUnreachable(t.to_string())),
    }
}

fn ws_str(ws: &Path) -> String {
    ws.display().to_string()
}
