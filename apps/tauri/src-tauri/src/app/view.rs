//! Views: the screen descriptions the frontend renders.

use std::path::Path;

use serde::Serialize;

use tasks_core::events::task::TaskEventKind;
use tasks_core::{Project, Status, Task};

use crate::daemon::TaskEvent;

/// A workspace summary for the header.
#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceView {
    /// The chosen working folder.
    pub root: String,
    /// The `.tasks` directory inside it.
    pub tasks_dir: String,
}

impl WorkspaceView {
    pub fn of(root: &Path) -> Self {
        Self {
            root: root.display().to_string(),
            tasks_dir: root.join(".tasks").display().to_string(),
        }
    }
}

/// A summarized event entry for the task detail changelog.
#[derive(Clone, Debug, Serialize)]
pub struct TaskEventView {
    pub id: String,
    pub created_at_millis: Option<u64>,
    /// Event type tag: "created", "renamed", "moved", "closed", "reopened",
    /// "description_updated", "status_changed", "snapshot".
    pub kind: String,
    /// Human-relevant detail (new name for renames, project id for moves, etc.).
    pub detail: Option<String>,
}

impl TaskEventView {
    pub fn from_event(event: &TaskEvent, statuses: &[Status]) -> Self {
        let (kind, detail) = match &event.kind {
            TaskEventKind::Created { name, .. } => ("created", Some(name.clone())),
            TaskEventKind::Renamed { new_name } => ("renamed", Some(new_name.clone())),
            TaskEventKind::Moved { new_project_id } => ("moved", Some(new_project_id.to_string())),
            TaskEventKind::Closed => ("closed", None),
            TaskEventKind::Reopened => ("reopened", None),
            TaskEventKind::DescriptionUpdated { description } => {
                ("description_updated", Some(description.clone()))
            }
            TaskEventKind::StatusChanged { status_id } => (
                "status_changed",
                status_id.and_then(|id| {
                    statuses.iter().find(|s| s.id == id).map(|s| s.name.clone())
                }),
            ),
            TaskEventKind::Snapshot { name, .. } => ("snapshot", Some(name.clone())),
        };
        Self {
            id: event.id.clone(),
            created_at_millis: event.created_at_millis,
            kind: kind.to_string(),
            detail,
        }
    }
}

/// The complete description of what to draw. The frontend switches on `screen`
/// and renders the carried data — it does no routing of its own.
///
/// Serialized as `{ "screen": "projects", "workspace": …, "projects": [...] }`.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "screen", rename_all = "snake_case")]
pub enum View {
    /// No workspace open: a full-screen "Select Repo" prompt.
    SelectRepo {
        /// Recently opened workspace paths (up to 3), most recent first.
        recent_workspaces: Vec<String>,
    },
    /// A workspace is open: its projects.
    Projects {
        workspace: WorkspaceView,
        projects: Vec<Project>,
    },
    /// A project is open: its tasks.
    Tasks {
        workspace: WorkspaceView,
        project: Project,
        projects: Vec<Project>,
        tasks: Vec<Task>,
        statuses: Vec<Status>,
    },
    /// A task is open: its detail and changelog.
    TaskDetail {
        workspace: WorkspaceView,
        project: Project,
        task: Task,
        events: Vec<TaskEventView>,
        statuses: Vec<Status>,
    },
}
