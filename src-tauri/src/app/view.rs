//! Views: the screen descriptions the frontend renders.

use serde::Serialize;

use crate::projects::Project;
use crate::shared::store::Workspace;
use crate::tasks::Task;

/// A workspace summary for the header.
#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceView {
    /// The chosen working folder.
    pub root: String,
    /// The `.tasks` directory inside it.
    pub tasks_dir: String,
}

impl WorkspaceView {
    pub fn of(workspace: &Workspace) -> Self {
        Self {
            root: workspace.root().display().to_string(),
            tasks_dir: workspace.dot_tasks().display().to_string(),
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
    SelectRepo,
    /// A workspace is open: its projects.
    Projects {
        workspace: WorkspaceView,
        projects: Vec<Project>,
    },
    /// A project is open: its tasks.
    Tasks {
        workspace: WorkspaceView,
        project: Project,
        tasks: Vec<Task>,
    },
}
