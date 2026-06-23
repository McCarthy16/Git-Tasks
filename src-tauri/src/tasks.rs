//! The tasks domain: the `Task` aggregate, its events, and the helpers to
//! create, load, and list tasks. Depends on the [`projects`](crate::projects)
//! domain — a task always belongs to an existing project.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::projects::{self, ProjectId};
use crate::shared::event::{Event, EventKind};
use crate::shared::id::prefixed_id;
use crate::shared::store::{self, Workspace};

/// The `.tasks` subfolder holding task event streams.
pub const COLLECTION: &str = "tasks";

prefixed_id!(
    /// Identifier for a task (`task_<hex>`).
    TaskId,
    "task_"
);

/// An event in a task's history.
///
/// Only `created` exists for now. The task↔project relationship lives in the
/// event payload (not the folder tree), so a task can later move between
/// projects via a future `moved` event without touching the file layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum TaskEventKind {
    Created { project_id: ProjectId, name: String },
}

impl EventKind for TaskEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            TaskEventKind::Created { .. } => "created",
        }
    }
}

/// A task event file.
pub type TaskEvent = Event<TaskEventKind>;

/// A task, as rebuilt by replaying its events.
#[derive(Clone, Debug, Serialize)]
pub struct Task {
    pub id: TaskId,
    /// The project this task belongs to.
    pub project_id: ProjectId,
    pub name: String,
    /// Creation time (ms since the Unix epoch), decoded from the `created` event.
    pub created_at_millis: Option<u64>,
}

impl Task {
    /// Rebuild a task by folding its events in chronological order.
    ///
    /// Returns `None` if the history does not start with a `created` event.
    pub fn replay(id: TaskId, events: &[TaskEvent]) -> Option<Task> {
        let mut task: Option<Task> = None;

        for event in events {
            match &event.kind {
                TaskEventKind::Created { project_id, name } => {
                    task = Some(Task {
                        id,
                        project_id: *project_id,
                        name: name.clone(),
                        created_at_millis: event.created_at_millis(),
                    });
                }
            }
        }

        task
    }
}

// --- Helpers: persistence for the tasks domain --------------------------

/// Create a task within an existing project, returning the rebuilt task.
///
/// Fails with [`Error::ProjectNotFound`] if `project_id` doesn't exist, so a
/// task can never reference a missing project.
pub fn create(ws: &Workspace, project_id: ProjectId, name: impl Into<String>) -> Result<Task> {
    if !projects::exists(ws, project_id)? {
        return Err(Error::ProjectNotFound(project_id));
    }
    let id = TaskId::new();
    let event = TaskEvent::new(TaskEventKind::Created {
        project_id,
        name: name.into(),
    });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    Task::replay(id, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Load a single task, or `None` if it has no events on disk.
pub fn load(ws: &Workspace, id: TaskId) -> Result<Option<Task>> {
    let events: Vec<TaskEvent> = store::read_all(&ws.events_dir(COLLECTION, id))?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(Task::replay(id, &events))
}

/// List every task in the workspace, oldest first.
pub fn list(ws: &Workspace) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    for id in store::list_ids::<TaskId>(&ws.collection_dir(COLLECTION))? {
        if let Some(task) = load(ws, id)? {
            tasks.push(task);
        }
    }
    Ok(tasks)
}

/// List the tasks belonging to `project_id`, oldest first.
pub fn list_in_project(ws: &Workspace, project_id: ProjectId) -> Result<Vec<Task>> {
    Ok(list(ws)?
        .into_iter()
        .filter(|task| task.project_id == project_id)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_lists_tasks() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let a = create(&ws, project.id, "First").unwrap();
        let b = create(&ws, project.id, "Second").unwrap();

        assert_eq!(a.project_id, project.id);

        let listed = list(&ws).unwrap();
        assert_eq!(listed.len(), 2);
        // UUIDv7 ordering => creation order.
        assert_eq!(listed[0].id, a.id);
        assert_eq!(listed[1].id, b.id);
    }

    #[test]
    fn requires_an_existing_project() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let err = create(&ws, ProjectId::new(), "Orphan").unwrap_err();
        assert!(matches!(err, Error::ProjectNotFound(_)));
        assert!(list(&ws).unwrap().is_empty());
    }

    #[test]
    fn list_in_project_filters_by_project() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let a = projects::create(&ws, "A").unwrap();
        let b = projects::create(&ws, "B").unwrap();
        create(&ws, a.id, "a1").unwrap();
        create(&ws, b.id, "b1").unwrap();
        create(&ws, a.id, "a2").unwrap();

        let a_tasks = list_in_project(&ws, a.id).unwrap();
        assert_eq!(a_tasks.len(), 2);
        assert!(a_tasks.iter().all(|t| t.project_id == a.id));
    }
}
