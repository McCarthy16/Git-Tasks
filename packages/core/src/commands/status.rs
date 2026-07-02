//! Status commands — the store-backed operations for the status entity.
//!
//! Statuses are workspace-level and ship with built-in *seeds* (see
//! [`storage::seeds`](crate::storage::seeds)). Seeds are never written to disk;
//! the storage layer overlays them at read time, so a command can rename or
//! remove a seed exactly like a user-created status — it just appends an
//! ordinary mutation event, and the seed snapshot is re-overlaid on the next
//! load. Reads here go through [`load_stream`]/[`stream_ids`] so seeds are
//! always present; the seed-blind [`reconstruction`](crate::reconstruction)
//! fold turns the assembled stream into a [`Status`].

use crate::error::{Error, Result};
use crate::events::status::{StatusEvent, StatusEventKind, COLLECTION};
use crate::events::store::EventStore;
use crate::projections::status::{Status, StatusId, StatusKind};
use crate::reconstruction::status::replay;
use crate::storage::seeds::{load_stream, stream_ids};

/// Create a user-defined status, returning the rebuilt status.
pub fn create(
    store: &impl EventStore,
    name: impl Into<String>,
    kind: StatusKind,
    description: Option<impl Into<String>>,
) -> Result<Status> {
    let id = StatusId::new();
    let event = StatusEvent::new(StatusEventKind::Created {
        name: name.into(),
        kind,
        description: description.map(Into::into),
    });
    store.append(COLLECTION, id, &event)?;
    replay(id, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Rename a status, returning the rebuilt status. Works for seeds and
/// user-created statuses alike.
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn rename(store: &impl EventStore, id: StatusId, new_name: impl Into<String>) -> Result<Status> {
    load(store, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::Renamed {
        new_name: new_name.into(),
    });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Update a status's description; pass `None` to clear it.
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn update_description(
    store: &impl EventStore,
    id: StatusId,
    description: Option<impl Into<String>>,
) -> Result<Status> {
    load(store, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::DescriptionUpdated {
        description: description.map(Into::into),
    });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Change the semantic kind of a status, returning the rebuilt status.
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn change_kind(store: &impl EventStore, id: StatusId, new_kind: StatusKind) -> Result<Status> {
    load(store, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::KindChanged { new_kind });
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Soft-remove a status. Its history is preserved; the status drops out of
/// [`list`] and shows up in [`list_removed`].
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn remove(store: &impl EventStore, id: StatusId) -> Result<Status> {
    load(store, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::Removed);
    store.append(COLLECTION, id, &event)?;
    load(store, id)?.ok_or(Error::NotCreated)
}

/// Load a single status, or `None` if it is neither a seed nor on disk.
///
/// Seeds are overlaid by [`load_stream`], so a seed with nothing written for it
/// still loads as its default.
pub fn load(store: &impl EventStore, id: StatusId) -> Result<Option<Status>> {
    let events = load_stream::<_, Status>(store, id)?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(replay(id, &events))
}

/// List active (non-removed) statuses: seeds first in canonical order, then
/// user-created ones oldest first.
pub fn list(store: &impl EventStore) -> Result<Vec<Status>> {
    Ok(all(store)?.into_iter().filter(|s| !s.removed).collect())
}

/// List removed statuses, in the same seeds-first order.
pub fn list_removed(store: &impl EventStore) -> Result<Vec<Status>> {
    Ok(all(store)?.into_iter().filter(|s| s.removed).collect())
}

/// Whether an active (non-removed) status with `id` exists.
pub fn exists(store: &impl EventStore, id: StatusId) -> Result<bool> {
    Ok(load(store, id)?.map(|s| !s.removed).unwrap_or(false))
}

/// Every status, removed or not, in canonical (seeds-first) order.
fn all(store: &impl EventStore) -> Result<Vec<Status>> {
    let mut result = Vec::new();
    for id in stream_ids::<_, Status>(store)? {
        if let Some(status) = load(store, id)? {
            result.push(status);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::seeds::{seed_id_for, Seeded};
    use crate::storage::FsEventStore;

    fn store() -> (tempfile::TempDir, FsEventStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = FsEventStore::new(tmp.path());
        (tmp, store)
    }

    #[test]
    fn seeds_present_without_any_disk_writes() {
        let (_tmp, store) = store();
        let active = list(&store).unwrap();
        assert_eq!(active.len(), Status::seed_slugs().len());
        assert_eq!(active[0].name, "Backlog");
        assert_eq!(active[0].kind, StatusKind::Unstarted);
        // A seed with nothing on disk carries no creation time.
        assert!(active[0].created_at_millis.is_none());
    }

    #[test]
    fn rename_seed_persists_and_replays_over_the_seed() {
        let (_tmp, store) = store();
        let backlog = seed_id_for::<Status>("backlog");

        let renamed = rename(&store, backlog, "Ice Box").unwrap();
        assert_eq!(renamed.name, "Ice Box");
        assert_eq!(renamed.id, backlog);

        // Seed order preserved; the mutation shows through on reload.
        let active = list(&store).unwrap();
        assert_eq!(active[0].id, backlog);
        assert_eq!(active[0].name, "Ice Box");
    }

    #[test]
    fn remove_hides_seed_from_list() {
        let (_tmp, store) = store();
        let todo = seed_id_for::<Status>("todo");

        remove(&store, todo).unwrap();

        assert!(!list(&store).unwrap().iter().any(|s| s.id == todo));
        let removed = list_removed(&store).unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].id, todo);
    }

    #[test]
    fn user_created_status_appears_after_seeds() {
        let (_tmp, store) = store();

        let custom = create(&store, "Needs QA", StatusKind::Started, None::<String>).unwrap();
        let active = list(&store).unwrap();
        assert_eq!(active.len(), Status::seed_slugs().len() + 1);
        assert_eq!(active.last().unwrap().id, custom.id);
        assert!(custom.created_at_millis.is_some());
    }

    #[test]
    fn update_description_and_change_kind_on_seed() {
        let (_tmp, store) = store();
        let canceled = seed_id_for::<Status>("canceled");

        let updated = update_description(&store, canceled, Some("Explicitly dropped")).unwrap();
        assert_eq!(updated.description.as_deref(), Some("Explicitly dropped"));

        let changed = change_kind(&store, canceled, StatusKind::Complete).unwrap();
        assert_eq!(changed.kind, StatusKind::Complete);
        // Clearing the description round-trips.
        let cleared = update_description(&store, canceled, None::<String>).unwrap();
        assert!(cleared.description.is_none());
    }

    #[test]
    fn exists_reflects_removal() {
        let (_tmp, store) = store();
        let backlog = seed_id_for::<Status>("backlog");
        assert!(exists(&store, backlog).unwrap());

        remove(&store, backlog).unwrap();
        assert!(!exists(&store, backlog).unwrap());

        // Unknown, non-seed id.
        assert!(!exists(&store, StatusId::new()).unwrap());
    }

    #[test]
    fn ops_on_unknown_id_error() {
        let (_tmp, store) = store();
        let unknown = StatusId::new();
        assert!(matches!(rename(&store, unknown, "x"), Err(Error::StatusNotFound(_))));
        assert!(matches!(remove(&store, unknown), Err(Error::StatusNotFound(_))));
    }
}
