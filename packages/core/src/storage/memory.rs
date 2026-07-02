//! An in-memory [`EventStore`] double for tests.
//!
//! [`InMemoryEventStore`] behaves exactly like [`FsEventStore`](super::fs::FsEventStore)
//! but keeps everything in a map instead of on disk — so tests run without
//! `tempfile` or filesystem I/O. It is gated behind the `test-util` feature (and
//! always compiled in for core's own tests), so it never ships in a production
//! build.
//!
//! Because [`EventStore`]'s methods are generic over the event payload `K`, the
//! store can't hold `Event<K>` for an arbitrary `K`; it type-erases each event
//! to its JSON form on append and re-parses it on read — the same round-trip the
//! filesystem store makes through disk. Events are keyed within a stream by
//! their `<id>-<type>` filename (as [`FsEventStore`](super::fs::FsEventStore)
//! names files), so a [`BTreeMap`] gives the same chronological ordering and the
//! same "re-append with the same id overwrites" semantics for free.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::Result;
use crate::events::envelope::{Event, EventKind};
use crate::events::store::EventStore;

/// A disk-free [`EventStore`], suitable for sharing across threads via `Arc`.
#[derive(Default)]
pub struct InMemoryEventStore {
    /// `(collection, entity id)` → (`<id>-<type>` filename → serialized event).
    streams: Mutex<HashMap<(String, String), BTreeMap<String, String>>>,
}

impl InMemoryEventStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventStore for InMemoryEventStore {
    fn append<K>(&self, collection: &str, id: impl Display, event: &Event<K>) -> Result<()>
    where
        K: Serialize + EventKind,
    {
        let filename = format!("{}-{}", event.id, event.kind.event_type());
        let json = serde_json::to_string(event)?;
        self.streams
            .lock()
            .expect("event store mutex poisoned")
            .entry((collection.to_string(), id.to_string()))
            .or_default()
            .insert(filename, json);
        Ok(())
    }

    fn read<K>(&self, collection: &str, id: impl Display) -> Result<Vec<Event<K>>>
    where
        K: DeserializeOwned,
    {
        let streams = self.streams.lock().expect("event store mutex poisoned");
        let Some(bucket) = streams.get(&(collection.to_string(), id.to_string())) else {
            return Ok(Vec::new());
        };
        // BTreeMap iterates in key order — i.e. chronological, since event ids
        // sort chronologically.
        let mut events = Vec::with_capacity(bucket.len());
        for json in bucket.values() {
            events.push(serde_json::from_str(json)?);
        }
        Ok(events)
    }

    fn list_ids<Id>(&self, collection: &str) -> Result<Vec<Id>>
    where
        Id: FromStr + Ord,
    {
        let streams = self.streams.lock().expect("event store mutex poisoned");
        let mut ids: Vec<Id> = streams
            .keys()
            .filter(|(c, _)| c == collection)
            .filter_map(|(_, id)| id.parse::<Id>().ok())
            .collect();
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::project::{self, ProjectEvent, ProjectEventKind};
    use crate::projections::project::ProjectId;

    fn created(name: &str) -> ProjectEvent {
        ProjectEvent::new(ProjectEventKind::Created { name: name.into() })
    }

    #[test]
    fn append_then_read_round_trips_in_order() {
        let store = InMemoryEventStore::new();
        let id = ProjectId::new();

        let first = created("Roadmap");
        let second = ProjectEvent::new(ProjectEventKind::Renamed {
            new_name: "Q3 Roadmap".into(),
        });
        store.append(project::COLLECTION, id, &first).unwrap();
        store.append(project::COLLECTION, id, &second).unwrap();

        let events: Vec<ProjectEvent> = store.read(project::COLLECTION, id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, first.id);
        assert_eq!(events[1].id, second.id);
    }

    #[test]
    fn re_appending_the_same_event_id_overwrites() {
        let store = InMemoryEventStore::new();
        let id = ProjectId::new();

        // Same event id + type => same "filename" => overwrite, not grow.
        let mut event = created("Draft");
        store.append(project::COLLECTION, id, &event).unwrap();
        if let ProjectEventKind::Created { name } = &mut event.kind {
            *name = "Final".into();
        }
        store.append(project::COLLECTION, id, &event).unwrap();

        let events: Vec<ProjectEvent> = store.read(project::COLLECTION, id).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0].kind, ProjectEventKind::Created { name } if name == "Final"));
    }

    #[test]
    fn reading_an_unknown_entity_yields_empty() {
        let store = InMemoryEventStore::new();
        let events: Vec<ProjectEvent> = store.read(project::COLLECTION, ProjectId::new()).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn list_ids_returns_entities_oldest_first_and_isolates_collections() {
        let store = InMemoryEventStore::new();
        let first = ProjectId::new();
        let second = ProjectId::new();
        store.append(project::COLLECTION, first, &created("A")).unwrap();
        store.append(project::COLLECTION, second, &created("B")).unwrap();

        let ids: Vec<ProjectId> = store.list_ids(project::COLLECTION).unwrap();
        assert_eq!(ids, vec![first, second]);
        // A different collection is empty — no bleed.
        assert!(store.list_ids::<ProjectId>("tasks").unwrap().is_empty());
    }

    // Proof the seam works: the exact same command logic that runs on
    // `FsEventStore` drives the in-memory store, seed overlay included.
    mod drives_commands {
        use super::*;
        use crate::commands::{project, status};
        use crate::projections::status::Status;
        use crate::storage::seeds::Seeded;

        #[test]
        fn project_create_and_reload() {
            let store = InMemoryEventStore::new();
            let created = project::create(&store, "Roadmap").unwrap();
            let reloaded = project::load(&store, created.id).unwrap().unwrap();
            assert_eq!(reloaded.name, "Roadmap");
        }

        #[test]
        fn status_seeds_and_overlay_work_over_memory() {
            let store = InMemoryEventStore::new();

            // Seeds are served with nothing written.
            let active = status::list(&store).unwrap();
            assert_eq!(active.len(), Status::seed_slugs().len());
            assert_eq!(active[0].name, "Backlog");

            // Renaming a seed overlays on reload, exactly as on disk.
            let renamed = status::rename(&store, active[0].id, "Future").unwrap();
            assert_eq!(renamed.name, "Future");
            assert_eq!(status::list(&store).unwrap()[0].name, "Future");
        }
    }
}
