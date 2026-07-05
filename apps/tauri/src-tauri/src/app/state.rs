//! The server-side UI state and its reducer.
//!
//! This holds app logic only: which workspace is open, what's selected, and
//! the recents list. All data operations are handed off to the tasks daemon
//! via the [`Daemon`] client — nothing here touches the event store directly.

use std::path::PathBuf;

use tasks_core::{EventId, ProjectId, StatusId, StatusKind, TaskId};

use crate::app::view::{TaskEventView, View, WorkspaceView};
use crate::daemon::Daemon;
use crate::error::{Error, Result};

const MAX_RECENTS: usize = 3;

/// The entire navigable state of the app, held server-side:
/// which workspace is open, and which project/task (if any) is selected. The
/// rendered [`View`] is derived from these fields plus data from the daemon.
#[derive(Default)]
pub struct AppState {
    daemon: Daemon,
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
    /// The open workspace root, or [`Error::NoWorkspace`].
    fn workspace(&self) -> Result<&PathBuf> {
        self.workspace.as_ref().ok_or(Error::NoWorkspace)
    }

    // --- Transitions ----------------------------------------------------

    /// Enter `root` as the workspace, asking the daemon to find-or-create its
    /// `.tasks` folder. Clears any selection and records the path in recents.
    pub fn open_workspace(&mut self, root: PathBuf) -> Result<()> {
        self.daemon.init_workspace(&root)?;
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
        self.daemon
            .update_task_description(self.workspace()?, task_id, description, None)?;
        Ok(())
    }

    /// Overwrite a specific description_updated event in place.
    pub fn update_task_description_in_place(
        &mut self,
        task_id: TaskId,
        event_id: EventId,
        description: String,
    ) -> Result<()> {
        self.daemon
            .update_task_description(self.workspace()?, task_id, description, Some(event_id))?;
        Ok(())
    }

    /// Overwrite a specific renamed event in place.
    pub fn rename_task_in_place(
        &mut self,
        task_id: TaskId,
        event_id: EventId,
        new_name: String,
    ) -> Result<()> {
        self.daemon
            .rename_task(self.workspace()?, task_id, new_name, Some(event_id))?;
        Ok(())
    }

    /// Create a project in the open workspace.
    pub fn create_project(&mut self, name: String) -> Result<()> {
        self.daemon.create_project(self.workspace()?, name)?;
        Ok(())
    }

    /// Rename a project.
    pub fn rename_project(&mut self, project_id: ProjectId, new_name: String) -> Result<()> {
        self.daemon.rename_project(self.workspace()?, project_id, new_name)?;
        Ok(())
    }

    /// Archive (close) a project.
    pub fn archive_project(&mut self, project_id: ProjectId) -> Result<()> {
        self.daemon.close_project(self.workspace()?, project_id)?;
        Ok(())
    }

    /// Restore (reopen) an archived project.
    pub fn restore_project(&mut self, project_id: ProjectId) -> Result<()> {
        self.daemon.reopen_project(self.workspace()?, project_id)?;
        Ok(())
    }

    /// Create a task in the open project.
    pub fn create_task(&mut self, name: String) -> Result<()> {
        let project_id = self.selected_project.ok_or(Error::NoProjectSelected)?;
        self.daemon.create_task(self.workspace()?, project_id, name)?;
        Ok(())
    }

    /// Rename a task.
    pub fn rename_task(&mut self, task_id: TaskId, new_name: String) -> Result<()> {
        self.daemon.rename_task(self.workspace()?, task_id, new_name, None)?;
        Ok(())
    }

    /// Move a task to a different project.
    pub fn move_task(&mut self, task_id: TaskId, project_id: ProjectId) -> Result<()> {
        self.daemon.move_task(self.workspace()?, task_id, project_id)?;
        Ok(())
    }

    /// Set or clear the status of a task.
    pub fn set_task_status(&mut self, task_id: TaskId, status_id: Option<StatusId>) -> Result<()> {
        self.daemon.set_task_status(self.workspace()?, task_id, status_id)?;
        Ok(())
    }

    /// Close a task.
    pub fn close_task(&mut self, task_id: TaskId) -> Result<()> {
        self.daemon.close_task(self.workspace()?, task_id)?;
        Ok(())
    }

    /// Reopen a closed task.
    pub fn reopen_task(&mut self, task_id: TaskId) -> Result<()> {
        self.daemon.reopen_task(self.workspace()?, task_id)?;
        Ok(())
    }

    /// Create a workflow status in the open workspace.
    pub fn create_status(
        &mut self,
        name: String,
        kind: StatusKind,
        description: Option<String>,
    ) -> Result<()> {
        self.daemon.create_status(self.workspace()?, name, kind, description)?;
        Ok(())
    }

    /// Rename a status.
    pub fn rename_status(&mut self, status_id: StatusId, new_name: String) -> Result<()> {
        self.daemon.rename_status(self.workspace()?, status_id, new_name)?;
        Ok(())
    }

    /// Update the description of a status (pass `None` to clear it).
    pub fn update_status_description(
        &mut self,
        status_id: StatusId,
        description: Option<String>,
    ) -> Result<()> {
        self.daemon.update_status_description(self.workspace()?, status_id, description)?;
        Ok(())
    }

    /// Change the semantic kind of a status.
    pub fn change_status_kind(&mut self, status_id: StatusId, new_kind: StatusKind) -> Result<()> {
        self.daemon.change_status_kind(self.workspace()?, status_id, new_kind)?;
        Ok(())
    }

    /// Soft-remove a status.
    pub fn remove_status(&mut self, status_id: StatusId) -> Result<()> {
        self.daemon.remove_status(self.workspace()?, status_id)?;
        Ok(())
    }

    /// Try to open the most recent workspace that still exists on disk.
    /// Silently does nothing if all recents are gone (or the daemon is down).
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
    /// project, fetching the latest data from the daemon.
    pub fn render(&self) -> Result<View> {
        let Some(ws) = &self.workspace else {
            return Ok(View::SelectRepo {
                recent_workspaces: self.recent_workspaces_strings(),
            });
        };
        let workspace = WorkspaceView::of(ws);

        // A selected project that still exists → its task list (or task detail).
        // Otherwise fall back to the project list.
        if let Some(project_id) = self.selected_project {
            if let Some(project) = self.daemon.load_project(ws, project_id)? {
                if let Some(task_id) = self.selected_task {
                    if let Some(task) = self.daemon.load_task(ws, task_id)? {
                        let raw_events = self.daemon.task_events(ws, task_id)?;
                        let all_statuses = self.daemon.list_statuses(ws)?;
                        let events = raw_events
                            .iter()
                            .map(|e| TaskEventView::from_event(e, &all_statuses))
                            .collect();
                        return Ok(View::TaskDetail {
                            workspace,
                            project,
                            projects: self.daemon.list_projects(ws)?,
                            task,
                            events,
                            statuses: all_statuses,
                        });
                    }
                    // Task not found — fall through to task list.
                }
                let tasks = self.daemon.list_tasks_in_project(ws, project_id)?;
                let projects = self.daemon.list_projects(ws)?;
                return Ok(View::Tasks {
                    workspace,
                    project,
                    projects,
                    tasks,
                    statuses: self.daemon.list_statuses(ws)?,
                });
            }
        }

        Ok(View::Projects {
            workspace,
            projects: self.daemon.list_projects(ws)?,
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
        assert!(matches!(state.render().unwrap(), View::SelectRepo { .. }));
    }

    /// End-to-end against a live daemon; run with `cargo test -- --ignored`
    /// while `tasks-server` is up.
    #[test]
    #[ignore = "requires a running tasks daemon on 127.0.0.1:4000"]
    fn walks_through_the_three_screens_via_the_daemon() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = AppState::default();

        state.open_workspace(tmp.path().to_path_buf()).unwrap();
        match state.render().unwrap() {
            View::Projects { projects, .. } => assert!(projects.is_empty()),
            other => panic!("expected projects, got {other:?}"),
        }

        state.create_project("Roadmap".into()).unwrap();
        let project_id = match state.render().unwrap() {
            View::Projects { projects, .. } => projects[0].id,
            other => panic!("expected projects, got {other:?}"),
        };
        state.open_project(project_id);
        state.create_task("First task".into()).unwrap();
        match state.render().unwrap() {
            View::Tasks { project, tasks, statuses, .. } => {
                assert_eq!(project.id, project_id);
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].name, "First task");
                assert!(!statuses.is_empty());
            }
            other => panic!("expected tasks, got {other:?}"),
        }

        let task_id = match state.render().unwrap() {
            View::Tasks { tasks, .. } => tasks[0].id,
            other => panic!("expected tasks, got {other:?}"),
        };
        state.open_task(task_id);
        match state.render().unwrap() {
            View::TaskDetail { task, events, .. } => {
                assert_eq!(task.id, task_id);
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].kind, "created");
            }
            other => panic!("expected task detail, got {other:?}"),
        }

        state.close_task_detail();
        state.close_project();
        state.close_workspace();
        assert!(matches!(state.render().unwrap(), View::SelectRepo { .. }));
    }
}
