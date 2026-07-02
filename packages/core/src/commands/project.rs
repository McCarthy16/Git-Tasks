//! Project commands — the store-backed operations for the project entity.
//!
//! Each write validates intent, appends a single event to the store, and
//! returns the freshly-rebuilt [`Project`]. The load/list reads that the
//! commands use to validate (and that callers use to serve views) live here
//! too, built on the [`EventStore`] trait and the seed-blind
//! [`reconstruction`](crate::reconstruction) fold.

use crate::error::{Error, Result};
use crate::events::project::{ProjectEvent, ProjectEventKind, COLLECTION};
use crate::events::store::EventStore;
use crate::projections::project::{Project, ProjectId};
use crate::reconstruction::project::replay;

/// Create a project by appending its `created` event, returning the rebuilt
/// project.
pub fn create(store: &impl EventStore, name: impl Into<String>) -> Result<Project> {
    let id = ProjectId::new();
    let event = ProjectEvent::new(ProjectEventKind::Created { name: name.into() });
    store.append(COLLECTION, id, &event)?;
    replay(id, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Rename a project, returning the rebuilt project.
///
/// Fails with [`Error::ProjectNotFound`] if the project doesn't exist.
pub fn rename(store: &impl EventStore, id: ProjectId, new_name: impl Into<String>) -> Result<Project> {
    load(store, id)?.ok_or(Error::ProjectNotFound(id))?;
    let event = ProjectEvent::new(ProjectEventKind::Renamed {
        new_name: new_name.into(),
    });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Close (archive) a project, returning the rebuilt project.
///
/// Fails with [`Error::ProjectNotFound`] if the project doesn't exist.
pub fn close(store: &impl EventStore, id: ProjectId) -> Result<Project> {
    load(store, id)?.ok_or(Error::ProjectNotFound(id))?;
    let event = ProjectEvent::new(ProjectEventKind::Closed);
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Reopen a closed project, returning the rebuilt project.
///
/// Fails with [`Error::ProjectNotFound`] if the project doesn't exist.
pub fn reopen(store: &impl EventStore, id: ProjectId) -> Result<Project> {
    load(store, id)?.ok_or(Error::ProjectNotFound(id))?;
    let event = ProjectEvent::new(ProjectEventKind::Reopened);
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Load a single project, or `None` if it has no events in the store.
pub fn load(store: &impl EventStore, id: ProjectId) -> Result<Option<Project>> {
    let events: Vec<ProjectEvent> = store.read(COLLECTION, id)?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(replay(id, &events))
}

/// List every open (non-closed) project in the store, oldest first.
pub fn list(store: &impl EventStore) -> Result<Vec<Project>> {
    list_where(store, |p| !p.closed)
}

/// List every closed (archived) project in the store, oldest first.
pub fn list_closed(store: &impl EventStore) -> Result<Vec<Project>> {
    list_where(store, |p| p.closed)
}

fn list_where(store: &impl EventStore, pred: impl Fn(&Project) -> bool) -> Result<Vec<Project>> {
    let mut projects = Vec::new();
    for id in store.list_ids::<ProjectId>(COLLECTION)? {
        if let Some(project) = load(store, id)? {
            if pred(&project) {
                projects.push(project);
            }
        }
    }
    Ok(projects)
}

/// Whether a project with `id` exists in the store.
pub fn exists(store: &impl EventStore, id: ProjectId) -> Result<bool> {
    Ok(load(store, id)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::FsEventStore;

    fn store() -> (tempfile::TempDir, FsEventStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        (tmp, store)
    }

    #[test]
    fn creates_and_reloads_a_project() {
        let (_tmp, store) = store();

        let created = create(&store, "Roadmap").unwrap();
        assert_eq!(created.name, "Roadmap");
        assert!(!created.closed);
        assert!(created.created_at_millis.is_some());

        let reloaded = load(&store, created.id).unwrap().unwrap();
        assert_eq!(reloaded.id, created.id);
        assert_eq!(reloaded.name, "Roadmap");
    }

    #[test]
    fn rename_updates_name_and_replays() {
        let (_tmp, store) = store();

        let project = create(&store, "Original").unwrap();
        let renamed = rename(&store, project.id, "Updated").unwrap();
        assert_eq!(renamed.name, "Updated");
        assert_eq!(renamed.id, project.id);

        let reloaded = load(&store, project.id).unwrap().unwrap();
        assert_eq!(reloaded.name, "Updated");
    }

    #[test]
    fn close_and_reopen_toggle_closed_flag() {
        let (_tmp, store) = store();

        let project = create(&store, "Roadmap").unwrap();
        assert!(close(&store, project.id).unwrap().closed);
        assert!(!reopen(&store, project.id).unwrap().closed);
        assert!(!load(&store, project.id).unwrap().unwrap().closed);
    }

    #[test]
    fn list_excludes_closed_and_list_closed_shows_them() {
        let (_tmp, store) = store();

        let a = create(&store, "Open").unwrap();
        let b = create(&store, "Archived").unwrap();
        close(&store, b.id).unwrap();

        let open = list(&store).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, a.id);

        let closed = list_closed(&store).unwrap();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id, b.id);
    }

    #[test]
    fn ops_on_missing_project_error() {
        let (_tmp, store) = store();
        let missing = ProjectId::new();
        assert!(matches!(rename(&store, missing, "x"), Err(Error::ProjectNotFound(_))));
        assert!(matches!(close(&store, missing), Err(Error::ProjectNotFound(_))));
        assert!(!exists(&store, missing).unwrap());
        assert!(load(&store, missing).unwrap().is_none());
    }
}
