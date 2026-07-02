//! The filesystem-backed [`EventStore`] implementation.
//!
//! [`FsEventStore`] persists events under a workspace's `.tasks` directory, one
//! file per event, laid out as:
//!
//! ```text
//! <root>/.tasks/
//!   projects/project_<hex>/events/<event-hex>-<type>.json
//!   tasks/task_<hex>/events/<event-hex>-<type>.json
//!   statuses/status_<hex>/events/<event-hex>-<type>.json
//! ```
//!
//! Each event is written once to a uniquely-named file. The filename is
//! `<event-hex>-<type>.json`: the UUIDv7 hex prefix keeps writes unique and,
//! because UUIDv7 sorts chronologically, sorting filenames replays events in
//! the order they happened. The `type` suffix makes the store self-describing on
//! disk. Events are never mutated, so concurrent contributors never collide.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::Result;
use crate::events::envelope::{Event, EventKind};
use crate::events::store::EventStore;

/// A workspace-rooted event store: a chosen repository folder and the `.tasks`
/// event log inside it.
///
/// Cheap to construct and clone-free to pass by reference — it holds only the
/// root path. Nothing is created on disk until the first event is appended (or
/// [`ensure`](Self::ensure) is called).
pub struct FsEventStore {
    root: PathBuf,
}

impl FsEventStore {
    /// Open a store rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The chosen working folder.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The `.tasks` directory holding all event data.
    pub fn dot_tasks(&self) -> PathBuf {
        self.root.join(".tasks")
    }

    /// The directory holding a collection's entities, e.g. `.tasks/projects`.
    fn collection_dir(&self, collection: &str) -> PathBuf {
        self.dot_tasks().join(collection)
    }

    /// The events directory for one entity, e.g.
    /// `.tasks/projects/project_<hex>/events`.
    fn events_dir(&self, collection: &str, id: impl Display) -> PathBuf {
        self.collection_dir(collection)
            .join(id.to_string())
            .join("events")
    }

    /// Find-or-create the `.tasks` tree for the given collections. Idempotent —
    /// safe to call on every launch whether or not the folders already exist.
    pub fn ensure(&self, collections: &[&str]) -> Result<()> {
        for collection in collections {
            fs::create_dir_all(self.collection_dir(collection))?;
        }
        Ok(())
    }
}

impl EventStore for FsEventStore {
    fn append<K>(&self, collection: &str, id: impl Display, event: &Event<K>) -> Result<()>
    where
        K: Serialize + EventKind,
    {
        let events_dir = self.events_dir(collection, id);
        fs::create_dir_all(&events_dir)?;
        let path = events_dir.join(format!("{}-{}.json", event.id, event.kind.event_type()));
        fs::write(path, serde_json::to_string_pretty(event)?)?;
        Ok(())
    }

    fn read<K>(&self, collection: &str, id: impl Display) -> Result<Vec<Event<K>>>
    where
        K: DeserializeOwned,
    {
        let events_dir = self.events_dir(collection, id);
        let mut files = match fs::read_dir(&events_dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                .collect::<Vec<_>>(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        files.sort();

        let mut events = Vec::with_capacity(files.len());
        for path in files {
            events.push(serde_json::from_str(&fs::read_to_string(path)?)?);
        }
        Ok(events)
    }

    fn list_ids<Id>(&self, collection: &str) -> Result<Vec<Id>>
    where
        Id: FromStr + Ord,
    {
        let entries = match fs::read_dir(self.collection_dir(collection)) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };

        let mut ids = Vec::new();
        for entry in entries.filter_map(|e| e.ok()) {
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(id) = name.parse::<Id>() {
                    ids.push(id);
                }
            }
        }
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::project::{self, ProjectEvent, ProjectEventKind};
    use crate::events::status::{self, StatusEvent, StatusEventKind};
    use crate::events::task::{self, TaskEvent, TaskEventKind};
    use crate::projections::project::ProjectId;
    use crate::projections::status::{StatusId, StatusKind};
    use crate::projections::task::TaskId;

    const COLLECTION: &str = project::COLLECTION;

    fn created(name: &str) -> ProjectEvent {
        ProjectEvent::new(ProjectEventKind::Created { name: name.into() })
    }

    #[test]
    fn ensure_finds_or_creates_collections() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());

        assert!(!store.dot_tasks().exists());
        store.ensure(&["projects", "tasks"]).unwrap();
        assert!(store.dot_tasks().join("projects").is_dir());
        assert!(store.dot_tasks().join("tasks").is_dir());

        // Idempotent.
        store.ensure(&["projects", "tasks"]).unwrap();
        assert!(store.dot_tasks().join("projects").is_dir());
    }

    #[test]
    fn append_then_read_round_trips_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        let id = ProjectId::new();

        let first = created("Roadmap");
        let second = ProjectEvent::new(ProjectEventKind::Renamed {
            new_name: "Q3 Roadmap".into(),
        });
        store.append(COLLECTION, id, &first).unwrap();
        store.append(COLLECTION, id, &second).unwrap();

        let events: Vec<ProjectEvent> = store.read(COLLECTION, id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, first.id);
        assert_eq!(events[1].id, second.id);
    }

    #[test]
    fn append_names_the_file_by_type() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        let id = ProjectId::new();

        store.append(COLLECTION, id, &created("Roadmap")).unwrap();

        let events_dir = store.dot_tasks().join(COLLECTION).join(id.to_string()).join("events");
        let files: Vec<_> = fs::read_dir(&events_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("-created.json"), "got {}", files[0]);
    }

    #[test]
    fn reading_an_unknown_entity_yields_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        let events: Vec<ProjectEvent> = store.read(COLLECTION, ProjectId::new()).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn list_ids_returns_entities_oldest_first() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());

        let first = ProjectId::new();
        let second = ProjectId::new();
        store.append(COLLECTION, first, &created("A")).unwrap();
        store.append(COLLECTION, second, &created("B")).unwrap();

        let ids: Vec<ProjectId> = store.list_ids(COLLECTION).unwrap();
        assert_eq!(ids, vec![first, second]);
    }

    #[test]
    fn list_ids_on_absent_collection_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        let ids: Vec<ProjectId> = store.list_ids(COLLECTION).unwrap();
        assert!(ids.is_empty());
    }

    /// The same store serves every entity: tasks and statuses use their own
    /// collections and ID types and never bleed into the projects collection.
    #[test]
    fn one_store_handles_projects_tasks_and_statuses() {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        store.ensure(crate::events::COLLECTIONS).unwrap();

        let project_id = ProjectId::new();
        let task_id = TaskId::new();
        let status_id = StatusId::new();

        store
            .append(project::COLLECTION, project_id, &created("Roadmap"))
            .unwrap();
        store
            .append(
                task::COLLECTION,
                task_id,
                &TaskEvent::new(TaskEventKind::Created {
                    project_id,
                    name: "Design the store".into(),
                }),
            )
            .unwrap();
        store
            .append(
                status::COLLECTION,
                status_id,
                &StatusEvent::new(StatusEventKind::Created {
                    name: "In Review".into(),
                    kind: StatusKind::Started,
                    description: None,
                }),
            )
            .unwrap();

        // Each stream reads back under its own typed event kind.
        let projects: Vec<ProjectEvent> = store.read(project::COLLECTION, project_id).unwrap();
        let tasks: Vec<TaskEvent> = store.read(task::COLLECTION, task_id).unwrap();
        let statuses: Vec<StatusEvent> = store.read(status::COLLECTION, status_id).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(tasks.len(), 1);
        assert_eq!(statuses.len(), 1);

        // Collections stay isolated — one entity each, no cross-contamination.
        assert_eq!(store.list_ids::<ProjectId>(project::COLLECTION).unwrap(), vec![project_id]);
        assert_eq!(store.list_ids::<TaskId>(task::COLLECTION).unwrap(), vec![task_id]);
        assert_eq!(store.list_ids::<StatusId>(status::COLLECTION).unwrap(), vec![status_id]);
    }
}
