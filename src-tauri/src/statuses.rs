//! The statuses domain: named workflow states with a semantic kind.
//!
//! Statuses are workspace-level. A fixed set of *seed* statuses always exists
//! without needing a `created` event on disk — they are the baseline every
//! workspace starts with. Users can rename, update, or remove seeds exactly
//! like user-created statuses; those changes are appended as normal events.
//!
//! ## Seed pattern (reusable for other domains)
//!
//! 1. Define seeds as a canonical `&[SeedDef]` with stable slugs.
//! 2. `seed_id(slug)` → deterministic [`StatusId`] via UUID v5.
//! 3. `replay(id, base, events)` — `base` is the seed default; events mutate
//!    it just like any user-created entity.
//! 4. `load()` injects the seed `base` before reading disk events.
//! 5. `list()` / `list_removed()` merge seeds-without-events + disk entities.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::shared::event::{Event, EventKind};
use crate::shared::id::{prefixed_id, seed_id};
use crate::shared::store::{self, Workspace};

/// The `.tasks` subfolder holding status event streams.
pub const COLLECTION: &str = "statuses";

prefixed_id!(
    /// Identifier for a status (`status_<hex>`).
    StatusId,
    "status_"
);

/// The semantic role of a status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
    Unstarted,
    Started,
    Complete,
    Canceled,
}

/// An event in a status's history.
///
/// Seed statuses never have a `Created` event — their history starts empty and
/// only accumulates update/remove events. User-created statuses always start
/// with a `Created` event.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum StatusEventKind {
    /// Only written for user-created statuses; seeds have no created event.
    Created {
        name: String,
        kind: StatusKind,
        description: Option<String>,
    },
    Renamed {
        new_name: String,
    },
    DescriptionUpdated {
        description: Option<String>,
    },
    KindChanged {
        new_kind: StatusKind,
    },
    Removed,
}

impl EventKind for StatusEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            StatusEventKind::Created { .. } => "created",
            StatusEventKind::Renamed { .. } => "renamed",
            StatusEventKind::DescriptionUpdated { .. } => "description_updated",
            StatusEventKind::KindChanged { .. } => "kind_changed",
            StatusEventKind::Removed => "removed",
        }
    }
}

/// A status event file.
pub type StatusEvent = Event<StatusEventKind>;

/// A status, as rebuilt by replaying its events (or returned as a seed default).
#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub id: StatusId,
    pub name: String,
    pub kind: StatusKind,
    pub description: Option<String>,
    pub removed: bool,
    /// `None` for seed statuses that have never been written to disk.
    pub created_at_millis: Option<u64>,
}

impl Status {
    /// Rebuild a status from an optional seed baseline plus its event history.
    ///
    /// - `base = Some(seed)` — start from the seed default; events may mutate it.
    /// - `base = None` — the first event must be `Created`; otherwise returns `None`.
    pub fn replay(id: StatusId, base: Option<Status>, events: &[StatusEvent]) -> Option<Status> {
        let mut status = base;

        for event in events {
            match &event.kind {
                StatusEventKind::Created { name, kind, description } => {
                    status = Some(Status {
                        id,
                        name: name.clone(),
                        kind: kind.clone(),
                        description: description.clone(),
                        removed: false,
                        created_at_millis: event.created_at_millis(),
                    });
                }
                StatusEventKind::Renamed { new_name } => {
                    if let Some(s) = status.as_mut() {
                        s.name = new_name.clone();
                    }
                }
                StatusEventKind::DescriptionUpdated { description } => {
                    if let Some(s) = status.as_mut() {
                        s.description = description.clone();
                    }
                }
                StatusEventKind::KindChanged { new_kind } => {
                    if let Some(s) = status.as_mut() {
                        s.kind = new_kind.clone();
                    }
                }
                StatusEventKind::Removed => {
                    if let Some(s) = status.as_mut() {
                        s.removed = true;
                    }
                }
            }
        }

        status
    }
}

// ---------------------------------------------------------------------------
// Seed definitions
// ---------------------------------------------------------------------------

/// Canonical slug ordering for the built-in statuses.
/// Seeds appear in this order before any user-created statuses.
const SEED_SLUGS: &[&str] = &[
    "backlog",
    "todo",
    "in_progress",
    "in_review",
    "complete",
    "canceled",
];

struct SeedDef {
    slug: &'static str,
    name: &'static str,
    kind: StatusKind,
}

fn seed_defs() -> &'static [SeedDef] {
    &[
        SeedDef { slug: "backlog",     name: "Backlog",     kind: StatusKind::Unstarted },
        SeedDef { slug: "todo",        name: "Todo",        kind: StatusKind::Unstarted },
        SeedDef { slug: "in_progress", name: "In Progress", kind: StatusKind::Started   },
        SeedDef { slug: "in_review",   name: "In Review",   kind: StatusKind::Started   },
        SeedDef { slug: "complete",    name: "Complete",    kind: StatusKind::Complete  },
        SeedDef { slug: "canceled",    name: "Canceled",    kind: StatusKind::Canceled  },
    ]
}

/// Returns a map of `StatusId → default Status` for every seed.
fn seeds_map() -> HashMap<StatusId, Status> {
    seed_defs()
        .iter()
        .map(|def| {
            let id: StatusId = seed_id(def.slug);
            let status = Status {
                id,
                name: def.name.to_string(),
                kind: def.kind.clone(),
                description: None,
                removed: false,
                created_at_millis: None,
            };
            (id, status)
        })
        .collect()
}

/// The deterministic ID for a seed status given its slug.
pub fn status_seed_id(slug: &str) -> StatusId {
    seed_id(slug)
}

// ---------------------------------------------------------------------------
// Helpers: persistence for the statuses domain
// ---------------------------------------------------------------------------

/// Create a user-defined status, returning the rebuilt status.
pub fn create(
    ws: &Workspace,
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
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    Status::replay(id, None, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Rename a status, returning the rebuilt status.
///
/// Works for both seed and user-created statuses.
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn rename(ws: &Workspace, id: StatusId, new_name: impl Into<String>) -> Result<Status> {
    load(ws, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::Renamed {
        new_name: new_name.into(),
    });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Update the description of a status. Pass `None` to clear it.
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn update_description(
    ws: &Workspace,
    id: StatusId,
    description: Option<impl Into<String>>,
) -> Result<Status> {
    load(ws, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::DescriptionUpdated {
        description: description.map(Into::into),
    });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Change the semantic kind of a status, returning the rebuilt status.
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn change_kind(ws: &Workspace, id: StatusId, new_kind: StatusKind) -> Result<Status> {
    load(ws, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::KindChanged { new_kind });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Soft-remove a status. The event history is preserved; the status is hidden
/// from [`list`] and visible via [`list_removed`].
///
/// Fails with [`Error::StatusNotFound`] if the status doesn't exist.
pub fn remove(ws: &Workspace, id: StatusId) -> Result<Status> {
    load(ws, id)?.ok_or(Error::StatusNotFound(id))?;
    let event = StatusEvent::new(StatusEventKind::Removed);
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    load(ws, id)?.ok_or(Error::NotCreated)
}

/// Load a single status, or `None` if it neither is a seed nor has events.
///
/// For seed IDs the seed default acts as the baseline; on-disk events are
/// replayed on top. For user-created IDs a `Created` event is required.
pub fn load(ws: &Workspace, id: StatusId) -> Result<Option<Status>> {
    let base = seeds_map().remove(&id);
    let events: Vec<StatusEvent> = store::read_all(&ws.events_dir(COLLECTION, id))?;
    if events.is_empty() && base.is_none() {
        return Ok(None);
    }
    Ok(Status::replay(id, base, &events))
}

/// List active (non-removed) statuses, seeds first then user-created.
pub fn list(ws: &Workspace) -> Result<Vec<Status>> {
    all(ws).map(|v| v.into_iter().filter(|s| !s.removed).collect())
}

/// List removed statuses, seeds first then user-created.
pub fn list_removed(ws: &Workspace) -> Result<Vec<Status>> {
    all(ws).map(|v| v.into_iter().filter(|s| s.removed).collect())
}

/// Whether an active (non-removed) status with `id` exists.
pub fn exists(ws: &Workspace, id: StatusId) -> Result<bool> {
    Ok(load(ws, id)?.map(|s| !s.removed).unwrap_or(false))
}

/// Collect every status: seeds (in canonical order) then user-created (oldest first).
fn all(ws: &Workspace) -> Result<Vec<Status>> {
    let disk_ids: Vec<StatusId> =
        store::list_ids::<StatusId>(&ws.collection_dir(COLLECTION))?;
    let disk_id_set: HashSet<StatusId> = disk_ids.iter().copied().collect();
    let mut seen: HashSet<StatusId> = HashSet::new();
    let mut result = Vec::new();

    // Seeds first, in SEED_SLUGS canonical order.
    // If a seed has on-disk events, replay them on top of its default.
    let mut sm = seeds_map();
    for &slug in SEED_SLUGS {
        let id: StatusId = seed_id(slug);
        seen.insert(id);
        let base = sm.remove(&id);
        let events: Vec<StatusEvent> = if disk_id_set.contains(&id) {
            store::read_all(&ws.events_dir(COLLECTION, id))?
        } else {
            vec![]
        };
        if let Some(status) = Status::replay(id, base, &events) {
            result.push(status);
        }
    }

    // User-created statuses — anything on disk that isn't a seed.
    for id in &disk_ids {
        if seen.contains(id) {
            continue;
        }
        if let Some(status) = load(ws, *id)? {
            result.push(status);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_present_without_any_disk_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let active = list(&ws).unwrap();
        assert_eq!(active.len(), 6);
        assert_eq!(active[0].name, "Backlog");
        assert_eq!(active[0].kind, StatusKind::Unstarted);
        assert_eq!(active[3].name, "In Review");
        assert_eq!(active[4].name, "Complete");
        assert_eq!(active[5].name, "Canceled");
    }

    #[test]
    fn seed_ids_are_deterministic_and_stable() {
        let id_a: StatusId = seed_id("backlog");
        let id_b: StatusId = seed_id("backlog");
        assert_eq!(id_a, id_b);

        let id_c: StatusId = seed_id("todo");
        assert_ne!(id_a, id_c);
    }

    #[test]
    fn rename_seed_persists_and_replays() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let backlog_id: StatusId = seed_id("backlog");
        rename(&ws, backlog_id, "Ice Box").unwrap();

        let active = list(&ws).unwrap();
        assert_eq!(active[0].name, "Ice Box"); // seed order preserved
        assert_eq!(active[0].id, backlog_id);

        // Reload just the one status.
        let reloaded = load(&ws, backlog_id).unwrap().unwrap();
        assert_eq!(reloaded.name, "Ice Box");
    }

    #[test]
    fn remove_seed_hides_from_list() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let todo_id: StatusId = seed_id("todo");
        remove(&ws, todo_id).unwrap();

        let active = list(&ws).unwrap();
        assert!(!active.iter().any(|s| s.id == todo_id));
        assert_eq!(active.len(), 5);

        let removed = list_removed(&ws).unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].id, todo_id);
    }

    #[test]
    fn user_created_status_appears_after_seeds() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let custom = create(&ws, "Needs QA", StatusKind::Started, None::<String>).unwrap();

        let active = list(&ws).unwrap();
        assert_eq!(active.len(), 7);
        // First 6 are seeds; custom is last.
        assert_eq!(active[6].id, custom.id);
    }

    #[test]
    fn update_description_and_change_kind_on_seed() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let id: StatusId = seed_id("canceled");
        let updated = update_description(&ws, id, Some("Explicitly dropped")).unwrap();
        assert_eq!(updated.description.as_deref(), Some("Explicitly dropped"));

        let changed = change_kind(&ws, id, StatusKind::Complete).unwrap();
        assert_eq!(changed.kind, StatusKind::Complete);
    }

    #[test]
    fn ops_on_unknown_id_error() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let unknown = StatusId::new();
        assert!(matches!(rename(&ws, unknown, "x"), Err(Error::StatusNotFound(_))));
        assert!(matches!(remove(&ws, unknown), Err(Error::StatusNotFound(_))));
    }
}
