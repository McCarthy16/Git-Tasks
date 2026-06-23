//! The server-side UI state and its reducer.

use std::path::PathBuf;

use crate::app::view::{View, WorkspaceView};
use crate::error::{Error, Result};
use crate::projects::{self, ProjectId};
use crate::shared::store::Workspace;
use crate::tasks;

/// The entire navigable state of the app, held server-side:
/// which workspace is open, and which project (if any) is selected. The
/// rendered [`View`] is derived from these two fields plus what's on disk.
#[derive(Default)]
pub struct AppState {
    workspace: Option<PathBuf>,
    selected_project: Option<ProjectId>,
}

impl AppState {
    /// A [`Workspace`] for the open repo, if one is open.
    fn workspace(&self) -> Option<Workspace> {
        self.workspace.clone().map(Workspace::new)
    }

    // --- Transitions ----------------------------------------------------

    /// Enter `root` as the workspace, finding-or-creating its `.tasks` folder.
    /// Clears any selected project.
    pub fn open_workspace(&mut self, root: PathBuf) -> Result<()> {
        Workspace::new(root.clone()).ensure(&[projects::COLLECTION, tasks::COLLECTION])?;
        self.workspace = Some(root);
        self.selected_project = None;
        Ok(())
    }

    /// Leave the workspace, returning to the select-repo screen.
    pub fn close_workspace(&mut self) {
        self.workspace = None;
        self.selected_project = None;
    }

    /// Select a project, moving to its task list.
    pub fn open_project(&mut self, project_id: ProjectId) {
        self.selected_project = Some(project_id);
    }

    /// Deselect the project, returning to the project list.
    pub fn close_project(&mut self) {
        self.selected_project = None;
    }

    /// Create a project in the open workspace.
    pub fn create_project(&mut self, name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        projects::create(&ws, name)?;
        Ok(())
    }

    /// Create a task in the open project.
    pub fn create_task(&mut self, name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        let project_id = self.selected_project.ok_or(Error::NoProjectSelected)?;
        tasks::create(&ws, project_id, name)?;
        Ok(())
    }

    // --- Rendering ------------------------------------------------------

    /// Derive the current [`View`] from the open workspace and selected
    /// project, reading the latest data from disk.
    pub fn render(&self) -> Result<View> {
        let Some(ws) = self.workspace() else {
            return Ok(View::SelectRepo);
        };
        let workspace = WorkspaceView::of(&ws);

        // A selected project that still exists → its task list. Otherwise fall
        // back to the project list (covers a deleted/renamed project).
        if let Some(project_id) = self.selected_project {
            if let Some(project) = projects::load(&ws, project_id)? {
                let tasks = tasks::list_in_project(&ws, project_id)?;
                return Ok(View::Tasks {
                    workspace,
                    project,
                    tasks,
                });
            }
        }

        Ok(View::Projects {
            workspace,
            projects: projects::list(&ws)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boots_to_select_repo() {
        let state = AppState::default();
        assert!(matches!(state.render().unwrap(), View::SelectRepo));
    }

    #[test]
    fn walks_through_the_three_screens() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = AppState::default();

        // Open workspace → projects screen (empty).
        state.open_workspace(tmp.path().to_path_buf()).unwrap();
        match state.render().unwrap() {
            View::Projects { projects, .. } => assert!(projects.is_empty()),
            other => panic!("expected projects, got {other:?}"),
        }

        // Create a project, then open it → tasks screen.
        state.create_project("Roadmap".into()).unwrap();
        let project_id = match state.render().unwrap() {
            View::Projects { projects, .. } => {
                assert_eq!(projects.len(), 1);
                projects[0].id
            }
            other => panic!("expected projects, got {other:?}"),
        };
        state.open_project(project_id);
        state.create_task("First task".into()).unwrap();
        match state.render().unwrap() {
            View::Tasks { project, tasks, .. } => {
                assert_eq!(project.id, project_id);
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].name, "First task");
            }
            other => panic!("expected tasks, got {other:?}"),
        }

        // Back out to projects, then close the workspace.
        state.close_project();
        assert!(matches!(state.render().unwrap(), View::Projects { .. }));
        state.close_workspace();
        assert!(matches!(state.render().unwrap(), View::SelectRepo));
    }

    #[test]
    fn create_without_context_errors() {
        let mut state = AppState::default();
        assert!(matches!(
            state.create_project("x".into()),
            Err(Error::NoWorkspace)
        ));

        let tmp = tempfile::tempdir().unwrap();
        state.open_workspace(tmp.path().to_path_buf()).unwrap();
        assert!(matches!(
            state.create_task("x".into()),
            Err(Error::NoProjectSelected)
        ));
    }
}
