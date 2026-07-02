//! Task commands — the store-backed operations for the task entity.
//!
//! A task always belongs to a project, so [`create`] and [`move_to_project`]
//! validate the target project via the [`project`] commands, and
//! [`set_status`] validates the status via the [`status`] commands. Each write
//! appends a single event and returns the rebuilt
//! [`Task`]; the load/list reads used to validate them (and to serve views)
//! live here too.

use crate::commands::{project, status};
use crate::error::{Error, Result};
use crate::events::store::EventStore;
use crate::events::task::{TaskEvent, TaskEventKind, COLLECTION};
use crate::projections::project::ProjectId;
use crate::projections::status::StatusId;
use crate::projections::task::{Task, TaskId};
use crate::reconstruction::task::replay;
use crate::shared::id::EventId;

/// Create a task within an existing project, returning the rebuilt task.
///
/// Fails with [`Error::ProjectNotFound`] if `project_id` doesn't exist, so a
/// task can never reference a missing project.
pub fn create(
    store: &impl EventStore,
    project_id: ProjectId,
    name: impl Into<String>,
) -> Result<Task> {
    if !project::exists(store, project_id)? {
        return Err(Error::ProjectNotFound(project_id));
    }
    let id = TaskId::new();
    let event = TaskEvent::new(TaskEventKind::Created {
        project_id,
        name: name.into(),
    });
    store.append(COLLECTION, id, &event)?;
    replay(id, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Rename a task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn rename(store: &impl EventStore, id: TaskId, new_name: impl Into<String>) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Renamed {
        new_name: new_name.into(),
    });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Move a task to a different project, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] or [`Error::ProjectNotFound`] if either
/// entity doesn't exist.
pub fn move_to_project(
    store: &impl EventStore,
    id: TaskId,
    new_project_id: ProjectId,
) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    if !project::exists(store, new_project_id)? {
        return Err(Error::ProjectNotFound(new_project_id));
    }
    let event = TaskEvent::new(TaskEventKind::Moved { new_project_id });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Close a task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn close(store: &impl EventStore, id: TaskId) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Closed);
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Reopen a closed task, returning the rebuilt task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn reopen(store: &impl EventStore, id: TaskId) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::Reopened);
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Set (or clear, with `None`) a task's status, returning the rebuilt task.
///
/// A non-`None` status must exist and be active. Fails with
/// [`Error::TaskNotFound`] if the task is missing or [`Error::StatusNotFound`]
/// if the status is missing/removed.
pub fn set_status(
    store: &impl EventStore,
    id: TaskId,
    status_id: Option<StatusId>,
) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    if let Some(sid) = status_id {
        if !status::exists(store, sid)? {
            return Err(Error::StatusNotFound(sid));
        }
    }
    let event = TaskEvent::new(TaskEventKind::StatusChanged { status_id });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Update a task's description by appending a new event, returning the rebuilt
/// task.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn update_description(
    store: &impl EventStore,
    id: TaskId,
    description: impl Into<String>,
) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent::new(TaskEventKind::DescriptionUpdated {
        description: description.into(),
    });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Overwrite an existing `description_updated` event in place, identified by
/// `event_id`, returning the rebuilt task.
///
/// Because events are named on disk by their id and type, re-appending an event
/// with the same `event_id` overwrites that one file rather than growing the
/// stream. Used for session-scoped deduplication: the frontend passes back the
/// event it created this page visit so repeated saves collapse into one event.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn update_description_in_place(
    store: &impl EventStore,
    id: TaskId,
    event_id: EventId,
    description: impl Into<String>,
) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent {
        id: event_id,
        kind: TaskEventKind::DescriptionUpdated {
            description: description.into(),
        },
    };
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Overwrite an existing `renamed` event in place, identified by `event_id`,
/// returning the rebuilt task. The in-place counterpart to [`rename`], used for
/// session-scoped deduplication of inline title edits.
///
/// Fails with [`Error::TaskNotFound`] if the task doesn't exist.
pub fn rename_in_place(
    store: &impl EventStore,
    id: TaskId,
    event_id: EventId,
    new_name: impl Into<String>,
) -> Result<Task> {
    load(store, id)?.ok_or(Error::TaskNotFound(id))?;
    let event = TaskEvent {
        id: event_id,
        kind: TaskEventKind::Renamed {
            new_name: new_name.into(),
        },
    };
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Load the raw event history for a task, oldest first.
pub fn load_events(store: &impl EventStore, id: TaskId) -> Result<Vec<TaskEvent>> {
    store.read(COLLECTION, id)
}

/// Load a single task, or `None` if it has no events in the store.
pub fn load(store: &impl EventStore, id: TaskId) -> Result<Option<Task>> {
    let events: Vec<TaskEvent> = store.read(COLLECTION, id)?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(replay(id, &events))
}

/// List open (non-closed) tasks in the store, oldest first.
pub fn list(store: &impl EventStore) -> Result<Vec<Task>> {
    list_where(store, |t| !t.closed)
}

/// List closed tasks in the store, oldest first.
pub fn list_closed(store: &impl EventStore) -> Result<Vec<Task>> {
    list_where(store, |t| t.closed)
}

fn list_where(store: &impl EventStore, pred: impl Fn(&Task) -> bool) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    for id in store.list_ids::<TaskId>(COLLECTION)? {
        if let Some(task) = load(store, id)? {
            if pred(&task) {
                tasks.push(task);
            }
        }
    }
    Ok(tasks)
}

/// List open tasks belonging to `project_id`, oldest first.
pub fn list_in_project(store: &impl EventStore, project_id: ProjectId) -> Result<Vec<Task>> {
    Ok(list(store)?
        .into_iter()
        .filter(|task| task.project_id == project_id)
        .collect())
}

/// List closed tasks belonging to `project_id`, oldest first.
pub fn list_closed_in_project(
    store: &impl EventStore,
    project_id: ProjectId,
) -> Result<Vec<Task>> {
    Ok(list_closed(store)?
        .into_iter()
        .filter(|task| task.project_id == project_id)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::seeds::seed_id_for;
    use crate::storage::FsEventStore;

    fn store() -> (tempfile::TempDir, FsEventStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        (tmp, store)
    }

    #[test]
    fn creates_and_lists_tasks_in_creation_order() {
        let (_tmp, store) = store();
        let p = project::create(&store, "Roadmap").unwrap();
        let a = create(&store, p.id, "First").unwrap();
        let b = create(&store, p.id, "Second").unwrap();

        assert_eq!(a.project_id, p.id);
        let listed = list(&store).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, a.id);
        assert_eq!(listed[1].id, b.id);
    }

    #[test]
    fn create_requires_an_existing_project() {
        let (_tmp, store) = store();
        let err = create(&store, ProjectId::new(), "Orphan").unwrap_err();
        assert!(matches!(err, Error::ProjectNotFound(_)));
        assert!(list(&store).unwrap().is_empty());
    }

    #[test]
    fn move_changes_project_and_updates_lists() {
        let (_tmp, store) = store();
        let a = project::create(&store, "A").unwrap();
        let b = project::create(&store, "B").unwrap();
        let task = create(&store, a.id, "Task").unwrap();

        move_to_project(&store, task.id, b.id).unwrap();
        assert_eq!(load(&store, task.id).unwrap().unwrap().project_id, b.id);
        assert!(list_in_project(&store, a.id).unwrap().is_empty());
        assert_eq!(list_in_project(&store, b.id).unwrap().len(), 1);

        assert!(matches!(
            move_to_project(&store, task.id, ProjectId::new()),
            Err(Error::ProjectNotFound(_))
        ));
    }

    #[test]
    fn close_and_reopen_split_the_lists() {
        let (_tmp, store) = store();
        let p = project::create(&store, "Roadmap").unwrap();
        let a = create(&store, p.id, "Open").unwrap();
        let b = create(&store, p.id, "Done").unwrap();
        close(&store, b.id).unwrap();

        let open = list(&store).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, a.id);
        assert_eq!(list_closed(&store).unwrap()[0].id, b.id);
        assert_eq!(list_closed_in_project(&store, p.id).unwrap().len(), 1);

        assert!(!reopen(&store, b.id).unwrap().closed);
        assert_eq!(list(&store).unwrap().len(), 2);
    }

    #[test]
    fn set_status_assigns_a_seed_and_clears() {
        let (_tmp, store) = store();
        let p = project::create(&store, "Roadmap").unwrap();
        let task = create(&store, p.id, "Ship it").unwrap();
        assert!(task.status_id.is_none());

        // Seeds exist without any disk events, so set_status accepts them.
        let backlog = seed_id_for::<crate::projections::status::Status>("backlog");
        assert_eq!(set_status(&store, task.id, Some(backlog)).unwrap().status_id, Some(backlog));
        assert!(set_status(&store, task.id, None).unwrap().status_id.is_none());
    }

    #[test]
    fn set_status_rejects_unknown_status() {
        let (_tmp, store) = store();
        let p = project::create(&store, "Roadmap").unwrap();
        let task = create(&store, p.id, "Task").unwrap();
        assert!(matches!(
            set_status(&store, task.id, Some(StatusId::new())),
            Err(Error::StatusNotFound(_))
        ));
    }

    #[test]
    fn in_place_edits_overwrite_rather_than_grow_the_stream() {
        let (_tmp, store) = store();
        let p = project::create(&store, "Roadmap").unwrap();
        let task = create(&store, p.id, "Task").unwrap();

        let eid = EventId::new();
        update_description_in_place(&store, task.id, eid, "draft").unwrap();
        update_description_in_place(&store, task.id, eid, "final").unwrap();

        let events = load_events(&store, task.id).unwrap();
        // created + exactly one description event (the second overwrote the first).
        assert_eq!(events.len(), 2);
        assert_eq!(load(&store, task.id).unwrap().unwrap().description, "final");

        let reid = EventId::new();
        rename_in_place(&store, task.id, reid, "One").unwrap();
        rename_in_place(&store, task.id, reid, "Two").unwrap();
        let events = load_events(&store, task.id).unwrap();
        assert_eq!(events.len(), 3); // created + description + one rename
        assert_eq!(load(&store, task.id).unwrap().unwrap().name, "Two");
    }

    #[test]
    fn ops_on_missing_task_error() {
        let (_tmp, store) = store();
        let missing = TaskId::new();
        assert!(matches!(rename(&store, missing, "x"), Err(Error::TaskNotFound(_))));
        assert!(matches!(close(&store, missing), Err(Error::TaskNotFound(_))));
        assert!(load(&store, missing).unwrap().is_none());
    }
}
