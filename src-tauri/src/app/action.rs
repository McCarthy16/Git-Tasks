//! Actions: the intents the UI can dispatch.

use serde::Deserialize;

use crate::projects::ProjectId;

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
    /// Leave the workspace, returning to the select-repo screen.
    CloseWorkspace,
    /// Open a project's task list.
    OpenProject { project_id: ProjectId },
    /// Return from a project's task list to the project list.
    CloseProject,
    /// Create a project in the open workspace; stays on the project list.
    CreateProject { name: String },
    /// Create a task in the open project; stays on its task list.
    CreateTask { name: String },
}
