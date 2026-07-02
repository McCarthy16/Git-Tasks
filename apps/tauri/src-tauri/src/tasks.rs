//! The tasks domain: the `Task` aggregate, its events, and the helpers to
//! create, load, and list tasks. Depends on the [`projects`](crate::projects)
//! domain — a task always belongs to an existing project.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::projects::{self, ProjectId};
use crate::shared::event::{Event, EventKind};
use crate::shared::id::{prefixed_id, EventId};
use crate::shared::store::{self, Workspace};
use crate::statuses::{self, StatusId};

/// The `.tasks` subfolder holding task event streams.
pub const COLLECTION: &str = "tasks";

prefixed_id!(
    /// Identifier for a task (`task_<hex>`).
    TaskId,
    "task_"
);

/// An event in a task's history.
///
/// The task↔project relationship lives in the event payload (not the folder
/// tree), so `Moved` can change the owning project without touching file layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum TaskEventKind {
    Created { project_id: ProjectId, name: String },
    Renamed { new_name: String },
    Moved { new_project_id: ProjectId },
    Closed,
    Reopened,
    DescriptionUpdated { description: String },
    /// `None` means "no status" (explicitly cleared).
    StatusChanged { status_id: Option<StatusId> },
}

impl EventKind for TaskEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            TaskEventKind::Created { .. } => "created",
            TaskEventKind::Renamed { .. } => "renamed",
            TaskEventKind::Moved { .. } => "moved",
            TaskEventKind::Closed => "closed",
            TaskEventKind::Reopened => "reopened",
            TaskEventKind::DescriptionUpdated { .. } => "description_updated",
            TaskEventKind::StatusChanged { .. } => "status_changed",
        }
    }
}

/// A task event file.
pub type TaskEvent = Event<TaskEventKind>;

/// A task, as rebuilt by replaying its events.
#[derive(Clone, Debug, Serialize)]
pub struct Task {
    pub id: TaskId,
    /// The project this task currently belongs to (may change via `moved`).
    pub project_id: ProjectId,
    pub name: String,
    pub description: String,
    /// `None` means the task has no status assigned.
    pub status_id: Option<StatusId>,
    pub closed: bool,
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
                        description: String::new(),
                        status_id: None,
                        closed: false,
                        created_at_millis: event.created_at_millis(),
                    });
                }
                TaskEventKind::Renamed { new_name } => {
                    if let Some(t) = task.as_mut() {
                        t.name = new_name.clone();
                    }
                }
                TaskEventKind::Moved { new_project_id } => {
                    if let Some(t) = task.as_mut() {
                        t.project_id = *new_project_id;
                    }
                }
                TaskEventKind::Closed => {
                    if let Some(t) = task.as_mut() {
                        t.closed = true;
                    }
                }
                TaskEventKind::Reopened => {
                    if let Some(t) = task.as_mut() {
                        t.closed = false;
                    }
                }
                TaskEventKind::DescriptionUpdated { description } => {
                    if let Some(t) = task.as_mut() {
                        t.description = description.clone();
                    }
                }
                TaskEventKind::StatusChanged { status_id } => {
                    if let Some(t) = task.as_mut() {
                        t.status_id = *status_id;
                    }
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

/// Rename a task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn rename(ws: &Workspace, id: TaskId, new_name: impl Into<String>) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Renamed {
        new_name: new_name.into(),
    });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Move a task to a different project, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] or [`Error::ProjectNotFound`] if either
/// entity doesn't exist.
pub fn move_to_project(ws: &Workspace, id: TaskId, new_project_id: ProjectId) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    if !projects::exists(ws, new_project_id)? {
        return Err(Error::ProjectNotFound(new_project_id));
    }
    let event = TaskEvent::new(TaskEventKind::Moved { new_project_id });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Close a task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn close(ws: &Workspace, id: TaskId) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Closed);
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Reopen a closed task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn reopen(ws: &Workspace, id: TaskId) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Reopened);
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Set the status of a task, returning the rebuilt task.
///
/// Pass `None` to clear the status. Validates that a non-None status exists
/// and is not removed before writing the event.
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn set_status(ws: &Workspace, id: TaskId, status_id: Option<StatusId>) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    if let Some(sid) = status_id {
        if !statuses::exists(ws, sid)? {
            return Err(Error::StatusNotFound(sid));
        }
    }
    let event = TaskEvent::new(TaskEventKind::StatusChanged { status_id });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Update a task's description by appending a new event, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn update_description(ws: &Workspace, id: TaskId, description: impl Into<String>) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::DescriptionUpdated { description: description.into() });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Overwrite an existing `description_updated` event in place, identified by
/// `event_id`. Used for session-scoped deduplication: the frontend tracks which
/// event it created this page visit and passes it back here on subsequent saves.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn update_description_in_place(
    ws: &Workspace,
    id: TaskId,
    event_id: EventId,
    description: impl Into<String>,
) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let updated = TaskEvent {
        id: event_id,
        kind: TaskEventKind::DescriptionUpdated { description: description.into() },
    };
    let path = ws
        .events_dir(COLLECTION, id)
        .join(format!("{}-description_updated.json", event_id));
    std::fs::write(&path, serde_json::to_string_pretty(&updated)?)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Overwrite an existing `renamed` event in place, identified by `event_id`.
/// Used for session-scoped deduplication of inline title edits on the detail page.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn rename_in_place(
    ws: &Workspace,
    id: TaskId,
    event_id: EventId,
    new_name: impl Into<String>,
) -> Result<Task> {
    load(ws, id)?.ok_or(Error::TaskNotFound(id))?;
    let updated = TaskEvent {
        id: event_id,
        kind: TaskEventKind::Renamed { new_name: new_name.into() },
    };
    let path = ws
        .events_dir(COLLECTION, id)
        .join(format!("{}-renamed.json", event_id));
    std::fs::write(&path, serde_json::to_string_pretty(&updated)?)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Load the raw event history for a task, oldest first.
pub fn load_events(ws: &Workspace, id: TaskId) -> Result<Vec<TaskEvent>> {
    store::read_all(&ws.events_dir(COLLECTION, id))
}

/// Load a single task, or `None` if it has no events on disk.
pub fn load(ws: &Workspace, id: TaskId) -> Result<Option<Task>> {
    let events: Vec<TaskEvent> = store::read_all(&ws.events_dir(COLLECTION, id))?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(Task::replay(id, &events))
}

/// List open (non-closed) tasks in the workspace, oldest first.
pub fn list(ws: &Workspace) -> Result<Vec<Task>> {
    list_where(ws, |t| !t.closed)
}

/// List closed tasks in the workspace, oldest first.
pub fn list_closed(ws: &Workspace) -> Result<Vec<Task>> {
    list_where(ws, |t| t.closed)
}

fn list_where(ws: &Workspace, pred: impl Fn(&Task) -> bool) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    for id in store::list_ids::<TaskId>(&ws.collection_dir(COLLECTION))? {
        if let Some(task) = load(ws, id)? {
            if pred(&task) {
                tasks.push(task);
            }
        }
    }
    Ok(tasks)
}

/// List open tasks belonging to `project_id`, oldest first.
pub fn list_in_project(ws: &Workspace, project_id: ProjectId) -> Result<Vec<Task>> {
    Ok(list(ws)?
        .into_iter()
        .filter(|task| task.project_id == project_id)
        .collect())
}

/// List closed tasks belonging to `project_id`, oldest first.
pub fn list_closed_in_project(ws: &Workspace, project_id: ProjectId) -> Result<Vec<Task>> {
    Ok(list_closed(ws)?
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
    fn rename_updates_name_and_replays() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let task = create(&ws, project.id, "Original").unwrap();

        let renamed = rename(&ws, task.id, "Updated").unwrap();
        assert_eq!(renamed.name, "Updated");
        assert_eq!(renamed.id, task.id);

        // Full reload from disk reflects the rename.
        let reloaded = load(&ws, task.id).unwrap().unwrap();
        assert_eq!(reloaded.name, "Updated");
    }

    #[test]
    fn move_changes_project_and_updates_list() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let a = projects::create(&ws, "A").unwrap();
        let b = projects::create(&ws, "B").unwrap();
        let task = create(&ws, a.id, "Task").unwrap();
        assert_eq!(task.project_id, a.id);

        move_to_project(&ws, task.id, b.id).unwrap();

        let moved = load(&ws, task.id).unwrap().unwrap();
        assert_eq!(moved.project_id, b.id);

        assert!(list_in_project(&ws, a.id).unwrap().is_empty());
        assert_eq!(list_in_project(&ws, b.id).unwrap().len(), 1);
    }

    #[test]
    fn close_and_reopen_toggle_closed_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let task = create(&ws, project.id, "Task").unwrap();
        assert!(!task.closed);

        let closed = close(&ws, task.id).unwrap();
        assert!(closed.closed);

        let reopened = reopen(&ws, task.id).unwrap();
        assert!(!reopened.closed);

        // Full reload preserves the final state.
        let reloaded = load(&ws, task.id).unwrap().unwrap();
        assert!(!reloaded.closed);
    }

    #[test]
    fn list_excludes_closed_and_list_closed_shows_them() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let a = create(&ws, project.id, "Open").unwrap();
        let b = create(&ws, project.id, "Done").unwrap();
        close(&ws, b.id).unwrap();

        let open = list(&ws).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, a.id);

        let closed = list_closed(&ws).unwrap();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id, b.id);

        // list_in_project and list_closed_in_project mirror the same split.
        assert_eq!(list_in_project(&ws, project.id).unwrap().len(), 1);
        assert_eq!(list_closed_in_project(&ws, project.id).unwrap().len(), 1);
    }

    #[test]
    fn update_on_missing_task_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let missing = TaskId::new();
        assert!(matches!(
            rename(&ws, missing, "x"),
            Err(Error::TaskNotFound(_))
        ));
        assert!(matches!(
            close(&ws, missing),
            Err(Error::TaskNotFound(_))
        ));
    }

    #[test]
    fn set_status_assigns_and_clears() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let task = create(&ws, project.id, "Ship it").unwrap();
        assert!(task.status_id.is_none());

        // Seeds exist without any disk events, so set_status accepts them.
        let backlog_id: StatusId = crate::shared::id::seed_id("backlog");
        let with_status = set_status(&ws, task.id, Some(backlog_id)).unwrap();
        assert_eq!(with_status.status_id, Some(backlog_id));

        // Clear the status.
        let cleared = set_status(&ws, task.id, None).unwrap();
        assert!(cleared.status_id.is_none());

        // Full reload preserves the final state.
        let reloaded = load(&ws, task.id).unwrap().unwrap();
        assert!(reloaded.status_id.is_none());
    }

    #[test]
    fn set_status_rejects_unknown_status() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let project = projects::create(&ws, "Roadmap").unwrap();
        let task = create(&ws, project.id, "Task").unwrap();

        let ghost = StatusId::new(); // not a seed, not on disk
        assert!(matches!(
            set_status(&ws, task.id, Some(ghost)),
            Err(Error::StatusNotFound(_))
        ));
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
