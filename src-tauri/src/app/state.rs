//! The server-side UI state and its reducer.

use std::path::PathBuf;

use crate::app::view::{TaskEventView, View, WorkspaceView};
use crate::error::{Error, Result};
use crate::projects::{self, ProjectId};
use crate::shared::id::EventId;
use crate::shared::store::Workspace;
use crate::statuses::{self, StatusId, StatusKind};
use crate::tasks::{self, TaskId};

const MAX_RECENTS: usize = 3;

/// The entire navigable state of the app, held server-side:
/// which workspace is open, and which project (if any) is selected. The
/// rendered [`View`] is derived from these two fields plus what's on disk.
#[derive(Default)]
pub struct AppState {
    workspace: Option<PathBuf>,
    selected_project: Option<ProjectId>,
    selected_task: Option<TaskId>,
    recent_workspaces: Vec<PathBuf>,
    /// Where to persist the recents list; None means no persistence.
    recents_file: Option<PathBuf>,
}

impl AppState {
    /// Create a new state that persists the recents list to `recents_file`.
    pub fn with_recents_file(recents_file: PathBuf) -> Self {
        let recent_workspaces = load_recents(&recents_file);
        Self {
            recents_file: Some(recents_file),
            recent_workspaces,
            ..Default::default()
        }
    }
}

impl AppState {
    /// A [`Workspace`] for the open repo, if one is open.
    fn workspace(&self) -> Option<Workspace> {
        self.workspace.clone().map(Workspace::new)
    }

    // --- Transitions ----------------------------------------------------

    /// Enter `root` as the workspace, finding-or-creating its `.tasks` folder.
    /// Clears any selected project and records the path in the recents list.
    pub fn open_workspace(&mut self, root: PathBuf) -> Result<()> {
        Workspace::new(root.clone())
            .ensure(&[projects::COLLECTION, tasks::COLLECTION, statuses::COLLECTION])?;
        self.push_recent(root.clone());
        self.workspace = Some(root);
        self.selected_project = None;
        self.selected_task = None;
        Ok(())
    }

    fn push_recent(&mut self, path: PathBuf) {
        self.recent_workspaces.retain(|p| p != &path);
        self.recent_workspaces.insert(0, path);
        self.recent_workspaces.truncate(MAX_RECENTS);
        if let Some(f) = &self.recents_file {
            save_recents(f, &self.recent_workspaces);
        }
    }

    /// Leave the workspace, returning to the select-repo screen.
    pub fn close_workspace(&mut self) {
        self.workspace = None;
        self.selected_project = None;
        self.selected_task = None;
    }

    /// Select a project, moving to its task list.
    pub fn open_project(&mut self, project_id: ProjectId) {
        self.selected_project = Some(project_id);
        self.selected_task = None;
    }

    /// Deselect the project, returning to the project list.
    pub fn close_project(&mut self) {
        self.selected_project = None;
        self.selected_task = None;
    }

    /// Open a task's detail view.
    pub fn open_task(&mut self, task_id: TaskId) {
        self.selected_task = Some(task_id);
    }

    /// Return from task detail to the task list.
    pub fn close_task_detail(&mut self) {
        self.selected_task = None;
    }

    /// Update a task's markdown description (appends a new event).
    pub fn update_task_description(&mut self, task_id: TaskId, description: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::update_description(&ws, task_id, description)?;
        Ok(())
    }

    /// Overwrite a specific description_updated event in place.
    pub fn update_task_description_in_place(
        &mut self,
        task_id: TaskId,
        event_id: EventId,
        description: String,
    ) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::update_description_in_place(&ws, task_id, event_id, description)?;
        Ok(())
    }

    /// Overwrite a specific renamed event in place.
    pub fn rename_task_in_place(
        &mut self,
        task_id: TaskId,
        event_id: EventId,
        new_name: String,
    ) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::rename_in_place(&ws, task_id, event_id, new_name)?;
        Ok(())
    }

    /// Create a project in the open workspace.
    pub fn create_project(&mut self, name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        projects::create(&ws, name)?;
        Ok(())
    }

    /// Rename a project.
    pub fn rename_project(&mut self, project_id: ProjectId, new_name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        projects::rename(&ws, project_id, new_name)?;
        Ok(())
    }

    /// Archive (close) a project.
    pub fn archive_project(&mut self, project_id: ProjectId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        projects::close(&ws, project_id)?;
        Ok(())
    }

    /// Restore (reopen) an archived project.
    pub fn restore_project(&mut self, project_id: ProjectId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        projects::reopen(&ws, project_id)?;
        Ok(())
    }

    /// Create a task in the open project.
    pub fn create_task(&mut self, name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        let project_id = self.selected_project.ok_or(Error::NoProjectSelected)?;
        tasks::create(&ws, project_id, name)?;
        Ok(())
    }

    /// Rename a task.
    pub fn rename_task(&mut self, task_id: TaskId, new_name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::rename(&ws, task_id, new_name)?;
        Ok(())
    }

    /// Move a task to a different project.
    pub fn move_task(&mut self, task_id: TaskId, project_id: ProjectId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::move_to_project(&ws, task_id, project_id)?;
        Ok(())
    }

    /// Set or clear the status of a task.
    pub fn set_task_status(&mut self, task_id: TaskId, status_id: Option<StatusId>) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::set_status(&ws, task_id, status_id)?;
        Ok(())
    }

    /// Close a task.
    pub fn close_task(&mut self, task_id: TaskId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::close(&ws, task_id)?;
        Ok(())
    }

    /// Reopen a closed task.
    pub fn reopen_task(&mut self, task_id: TaskId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        tasks::reopen(&ws, task_id)?;
        Ok(())
    }

    /// Create a workflow status in the open workspace.
    pub fn create_status(
        &mut self,
        name: String,
        kind: StatusKind,
        description: Option<String>,
    ) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        statuses::create(&ws, name, kind, description)?;
        Ok(())
    }

    /// Rename a status.
    pub fn rename_status(&mut self, status_id: StatusId, new_name: String) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        statuses::rename(&ws, status_id, new_name)?;
        Ok(())
    }

    /// Update the description of a status (pass `None` to clear it).
    pub fn update_status_description(
        &mut self,
        status_id: StatusId,
        description: Option<String>,
    ) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        statuses::update_description(&ws, status_id, description)?;
        Ok(())
    }

    /// Change the semantic kind of a status.
    pub fn change_status_kind(&mut self, status_id: StatusId, new_kind: StatusKind) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        statuses::change_kind(&ws, status_id, new_kind)?;
        Ok(())
    }

    /// Soft-remove a status.
    pub fn remove_status(&mut self, status_id: StatusId) -> Result<()> {
        let ws = self.workspace().ok_or(Error::NoWorkspace)?;
        statuses::remove(&ws, status_id)?;
        Ok(())
    }

    /// Try to open the most recent workspace that still exists on disk.
    /// Silently does nothing if all recents are gone.
    pub fn try_open_most_recent(&mut self) {
        let candidates: Vec<PathBuf> = self.recent_workspaces.clone();
        for path in candidates {
            if path.exists() && self.open_workspace(path).is_ok() {
                return;
            }
        }
    }

    /// The recent workspace paths as display strings, most recent first.
    pub fn recent_workspaces_strings(&self) -> Vec<String> {
        self.recent_workspaces
            .iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    // --- Rendering ------------------------------------------------------

    /// Derive the current [`View`] from the open workspace and selected
    /// project, reading the latest data from disk.
    pub fn render(&self) -> Result<View> {
        let Some(ws) = self.workspace() else {
            return Ok(View::SelectRepo {
                recent_workspaces: self
                    .recent_workspaces
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
            });
        };
        let workspace = WorkspaceView::of(&ws);

        // A selected project that still exists → its task list (or task detail).
        // Otherwise fall back to the project list.
        if let Some(project_id) = self.selected_project {
            if let Some(project) = projects::load(&ws, project_id)? {
                if let Some(task_id) = self.selected_task {
                    if let Some(task) = tasks::load(&ws, task_id)? {
                        let raw_events = tasks::load_events(&ws, task_id)?;
                        let all_statuses = statuses::list(&ws)?;
                        let events = raw_events
                            .iter()
                            .map(|e| TaskEventView::from_event(e, &all_statuses))
                            .collect();
                        return Ok(View::TaskDetail {
                            workspace,
                            project,
                            task,
                            events,
                            statuses: all_statuses,
                        });
                    }
                    // Task not found — fall through to task list.
                }
                let task_list = tasks::list_in_project(&ws, project_id)?;
                let all_projects = projects::list(&ws)?;
                return Ok(View::Tasks {
                    workspace,
                    project,
                    projects: all_projects,
                    tasks: task_list,
                    statuses: statuses::list(&ws)?,
                });
            }
        }

        Ok(View::Projects {
            workspace,
            projects: projects::list(&ws)?,
        })
    }
}

fn load_recents(path: &PathBuf) -> Vec<PathBuf> {
    let Ok(data) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(paths) = serde_json::from_str::<Vec<String>>(&data) else {
        return vec![];
    };
    paths.into_iter().map(PathBuf::from).collect()
}

fn save_recents(path: &PathBuf, recents: &[PathBuf]) {
    let paths: Vec<String> = recents.iter().map(|p| p.display().to_string()).collect();
    if let Ok(data) = serde_json::to_string(&paths) {
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(path));
        let _ = std::fs::write(path, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boots_to_select_repo() {
        let state = AppState::default();
        assert!(matches!(
            state.render().unwrap(),
            View::SelectRepo { .. }
        ));
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
        assert!(matches!(state.render().unwrap(), View::SelectRepo { .. }));
    }

    #[test]
    fn recents_are_tracked_and_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = AppState::default();

        let dirs: Vec<PathBuf> = (0..4)
            .map(|i| {
                let p = tmp.path().join(format!("repo{i}"));
                std::fs::create_dir_all(&p).unwrap();
                p
            })
            .collect();

        for d in &dirs {
            state.open_workspace(d.clone()).unwrap();
            state.close_workspace();
        }

        match state.render().unwrap() {
            View::SelectRepo { recent_workspaces } => {
                assert_eq!(recent_workspaces.len(), 3);
                // most recent first
                assert!(recent_workspaces[0].contains("repo3"));
            }
            other => panic!("expected select_repo, got {other:?}"),
        }
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
