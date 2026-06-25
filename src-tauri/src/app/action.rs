//! Actions: the intents the UI can dispatch.

use serde::Deserialize;

use crate::projects::ProjectId;
use crate::shared::id::EventId;
use crate::statuses::{StatusId, StatusKind};
use crate::tasks::TaskId;

/// An intent sent from the UI. The app reduces it against the current
/// [`AppState`](super::state::AppState) and returns a fresh
/// [`View`](super::view::View).
///
/// Serialized as `{ "type": "open_project", "project_id": "project_…" }`.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Open the native folder picker and enter the chosen workspace.
    ///
    /// The picker itself is a platform concern resolved by the Tauri adapter;
    /// the resolved path is applied via
    /// [`AppState::open_workspace`](super::state::AppState::open_workspace).
    PickWorkspace,
    /// Open a workspace directly by path (used for recent-repos shortcuts).
    OpenWorkspace { path: String },
    /// Leave the workspace, returning to the select-repo screen.
    CloseWorkspace,
    /// Open a project's task list.
    OpenProject { project_id: ProjectId },
    /// Return from a project's task list to the project list.
    CloseProject,
    /// Create a project in the open workspace; stays on the project list.
    CreateProject { name: String },
    /// Rename an existing project.
    RenameProject { project_id: ProjectId, new_name: String },
    /// Archive (close) a project.
    ArchiveProject { project_id: ProjectId },
    /// Restore (reopen) an archived project.
    RestoreProject { project_id: ProjectId },
    /// Create a task in the open project; stays on its task list.
    CreateTask { name: String },
    /// Rename an existing task.
    RenameTask { task_id: TaskId, new_name: String },
    /// Move a task to a different project.
    MoveTask { task_id: TaskId, project_id: ProjectId },
    /// Close (complete/archive) a task.
    CloseTask { task_id: TaskId },
    /// Reopen a previously closed task.
    ReopenTask { task_id: TaskId },
    /// Open a task's detail view.
    OpenTask { task_id: TaskId },
    /// Return from a task's detail view to the task list.
    CloseTaskDetail,
    /// Update the markdown description of a task (appends a new event).
    UpdateTaskDescription { task_id: TaskId, description: String },
    /// Overwrite a specific description_updated event in place (session dedup).
    UpdateTaskDescriptionInPlace { task_id: TaskId, event_id: EventId, description: String },
    /// Overwrite a specific renamed event in place (session dedup).
    RenameTaskInPlace { task_id: TaskId, event_id: EventId, new_name: String },
    /// Set (or clear) the status of a task. Pass null `status_id` to remove it.
    SetTaskStatus { task_id: TaskId, status_id: Option<StatusId> },
    /// Create a workflow status in the open workspace.
    CreateStatus { name: String, kind: StatusKind, description: Option<String> },
    /// Rename an existing status.
    RenameStatus { status_id: StatusId, new_name: String },
    /// Update the description of a status (pass null to clear it).
    UpdateStatusDescription { status_id: StatusId, description: Option<String> },
    /// Change the semantic kind of a status.
    ChangeStatusKind { status_id: StatusId, new_kind: StatusKind },
    /// Soft-remove a status.
    RemoveStatus { status_id: StatusId },
}
