//! Seeds — built-in baselines the store serves without anything on disk.
//!
//! A seed is a default that exists even though nothing was ever written for it.
//! Rather than being a special case anywhere else, a seed is expressed as a
//! synthetic **snapshot event**: the storage layer *assembles* it into the
//! entity's stream at read time, and everything above (reconstruction, commands,
//! the app) folds that stream without knowing a seed was involved. Seeds are a
//! storage concept and live here — nothing else needs to care about them.
//!
//! Two halves:
//!
//! - The *engine* ([`Seeded`], [`seed_event`], [`seed_events`], [`seed_id_for`])
//!   — generic, entity-agnostic machinery. An entity opts in by implementing
//!   [`Seeded`] to declare which seeds it has and each one's snapshot payload.
//! - The *overlay* ([`load_stream`], [`stream_ids`]) — reads that layer the
//!   declared seeds over whatever an [`EventStore`] holds on disk.
//!
//! A seed event's id is derived from its slug with UUID **v5**, so it is stable
//! across builds and machines *and* carries no timestamp — which is why a seed's
//! reconstructed `created_at_millis` is `None` until a real (v7) event is
//! written on top of it.

use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::error::Result;
use crate::events::envelope::Event;
use crate::events::store::EventStore;
use crate::shared::id::seed_id;

/// An entity that ships with built-in seed defaults.
///
/// The entity owns *what* it seeds (the slugs and each default's snapshot) and
/// *where* its events live ([`COLLECTION`](Self::COLLECTION)); the free
/// functions here own *how* seed events are derived and overlaid.
pub trait Seeded: Sized {
    /// The entity's identifier type, derived deterministically from a slug.
    type Id: From<Uuid>;

    /// The entity's event payload type — its `Snapshot` variant carries the
    /// full state a seed default sets.
    type EventKind;

    /// The event-store collection the entity's streams live in.
    const COLLECTION: &'static str;

    /// The canonical slugs for this entity's seeds, in display order.
    fn seed_slugs() -> &'static [&'static str];

    /// The snapshot payload defining the default full state for one `slug`.
    fn seed_snapshot(slug: &str) -> Self::EventKind;
}

/// The deterministic entity ID for a seed of type `T`, given its slug.
pub fn seed_id_for<T: Seeded>(slug: &str) -> T::Id {
    seed_id(slug)
}

/// The synthetic snapshot event for a seed `slug`: a stable v5-id snapshot.
pub fn seed_event<T: Seeded>(slug: &str) -> Event<T::EventKind> {
    Event {
        id: seed_id(slug),
        kind: T::seed_snapshot(slug),
    }
}

/// Every seed event declared by `T`, in slug order.
pub fn seed_events<T: Seeded>() -> Vec<Event<T::EventKind>> {
    T::seed_slugs()
        .iter()
        .map(|&slug| seed_event::<T>(slug))
        .collect()
}

/// Read the full logical stream for a seeded entity `id`, seeds included.
///
/// If `id` is one of `T`'s seeds, that seed's synthetic snapshot event leads the
/// stream, followed by any real events written on top of it; otherwise the
/// stream is just what's on disk. The result folds like any other stream — the
/// caller never learns whether a seed was involved.
pub fn load_stream<S, T>(store: &S, id: T::Id) -> Result<Vec<Event<T::EventKind>>>
where
    S: EventStore,
    T: Seeded,
    T::Id: Display + PartialEq,
    T::EventKind: DeserializeOwned,
{
    let mut stream = Vec::new();
    if let Some(slug) = T::seed_slugs()
        .iter()
        .copied()
        .find(|&slug| seed_id_for::<T>(slug) == id)
    {
        stream.push(seed_event::<T>(slug));
    }
    stream.extend(store.read::<T::EventKind>(T::COLLECTION, &id)?);
    Ok(stream)
}

/// The IDs of every seeded entity: all declared seeds first, in canonical slug
/// order, then any on-disk entities that aren't seeds, oldest first.
///
/// A seed with nothing written for it still appears — that's the whole point.
pub fn stream_ids<S, T>(store: &S) -> Result<Vec<T::Id>>
where
    S: EventStore,
    T: Seeded,
    T::Id: FromStr + Ord + Eq + Hash + Copy,
{
    let mut ids = Vec::new();
    let mut seen: HashSet<T::Id> = HashSet::new();

    for &slug in T::seed_slugs() {
        let id: T::Id = seed_id_for::<T>(slug);
        if seen.insert(id) {
            ids.push(id);
        }
    }
    for id in store.list_ids::<T::Id>(T::COLLECTION)? {
        if seen.insert(id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A self-contained entity to exercise the engine without depending on any
    // real domain type.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    struct FakeId(Uuid);
    impl From<Uuid> for FakeId {
        fn from(u: Uuid) -> Self {
            FakeId(u)
        }
    }

    struct FakeKind {
        label: String,
    }

    struct Fake;
    impl Seeded for Fake {
        type Id = FakeId;
        type EventKind = FakeKind;
        const COLLECTION: &'static str = "fakes";
        fn seed_slugs() -> &'static [&'static str] {
            &["alpha", "beta"]
        }
        fn seed_snapshot(slug: &str) -> FakeKind {
            FakeKind { label: slug.to_uppercase() }
        }
    }

    #[test]
    fn seed_ids_are_deterministic_and_distinct() {
        assert_eq!(seed_id_for::<Fake>("alpha"), seed_id_for::<Fake>("alpha"));
        assert_ne!(seed_id_for::<Fake>("alpha"), seed_id_for::<Fake>("beta"));
    }

    #[test]
    fn one_snapshot_event_per_slug_with_no_timestamp() {
        let events = seed_events::<Fake>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind.label, "ALPHA");
        // v5 ids carry no timestamp, so seeds have no creation time.
        assert!(events[0].created_at_millis().is_none());
    }

    // The overlay, exercised end-to-end against the real `Status` seeds: assemble
    // the stream from an `FsEventStore`, then fold it with the (seed-blind)
    // reconstruction to prove callers never have to think about seeds.
    mod overlay {
        use super::*;
        use crate::events::status::{StatusEvent, StatusEventKind};
        use crate::projections::status::{Status, StatusId, StatusKind};
        use crate::reconstruction::status::replay;
        use crate::storage::fs::FsEventStore;

        #[test]
        fn seed_with_no_disk_events_is_served_from_thin_air() {
            let tmp = tempfile::tempdir().unwrap();
            let store = FsEventStore::new(tmp.path());
            let id = seed_id_for::<Status>("backlog");

            let stream = load_stream::<_, Status>(&store, id).unwrap();
            assert_eq!(stream.len(), 1, "just the synthetic seed snapshot");

            let status = replay(id, &stream).unwrap();
            assert_eq!(status.name, "Backlog");
            assert_eq!(status.kind, StatusKind::Unstarted);
            assert!(status.created_at_millis.is_none(), "v5 seed id => no timestamp");
        }

        #[test]
        fn real_events_layer_on_top_of_the_seed() {
            let tmp = tempfile::tempdir().unwrap();
            let store = FsEventStore::new(tmp.path());
            let id = seed_id_for::<Status>("backlog");

            // Renaming a seed writes an ordinary event; the seed itself is never
            // persisted, only re-overlaid on the next read.
            store
                .append(
                    Status::COLLECTION,
                    id,
                    &StatusEvent::new(StatusEventKind::Renamed { new_name: "Ice Box".into() }),
                )
                .unwrap();

            let stream = load_stream::<_, Status>(&store, id).unwrap();
            assert_eq!(stream.len(), 2, "seed snapshot, then the renamed event");
            let status = replay(id, &stream).unwrap();
            assert_eq!(status.name, "Ice Box");
        }

        #[test]
        fn non_seed_id_reads_straight_through() {
            let tmp = tempfile::tempdir().unwrap();
            let store = FsEventStore::new(tmp.path());
            let id = StatusId::new();
            store
                .append(
                    Status::COLLECTION,
                    id,
                    &StatusEvent::new(StatusEventKind::Created {
                        name: "Needs QA".into(),
                        kind: StatusKind::Started,
                        description: None,
                    }),
                )
                .unwrap();

            let stream = load_stream::<_, Status>(&store, id).unwrap();
            assert_eq!(stream.len(), 1, "no seed prepended for a user-created id");
            assert_eq!(replay(id, &stream).unwrap().name, "Needs QA");
        }

        #[test]
        fn stream_ids_lists_seeds_first_then_user_created() {
            let tmp = tempfile::tempdir().unwrap();
            let store = FsEventStore::new(tmp.path());

            // Empty store: every seed still shows up, in slug order.
            let seeds_only = stream_ids::<_, Status>(&store).unwrap();
            let expected_seeds: Vec<StatusId> = Status::seed_slugs()
                .iter()
                .map(|&s| seed_id_for::<Status>(s))
                .collect();
            assert_eq!(seeds_only, expected_seeds);

            // A user-created status appends after all seeds.
            let custom = StatusId::new();
            store
                .append(
                    Status::COLLECTION,
                    custom,
                    &StatusEvent::new(StatusEventKind::Created {
                        name: "Needs QA".into(),
                        kind: StatusKind::Started,
                        description: None,
                    }),
                )
                .unwrap();

            let ids = stream_ids::<_, Status>(&store).unwrap();
            assert_eq!(ids.len(), expected_seeds.len() + 1);
            assert_eq!(*ids.last().unwrap(), custom);
        }

        #[test]
        fn a_seed_with_disk_events_is_not_double_listed() {
            let tmp = tempfile::tempdir().unwrap();
            let store = FsEventStore::new(tmp.path());
            let backlog = seed_id_for::<Status>("backlog");
            store
                .append(
                    Status::COLLECTION,
                    backlog,
                    &StatusEvent::new(StatusEventKind::Renamed { new_name: "Ice Box".into() }),
                )
                .unwrap();

            let ids = stream_ids::<_, Status>(&store).unwrap();
            assert_eq!(ids.iter().filter(|&&i| i == backlog).count(), 1);
            assert_eq!(ids.len(), Status::seed_slugs().len(), "no extra entry for the seed's disk stream");
        }
    }
}
