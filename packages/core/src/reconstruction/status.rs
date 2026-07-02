//! Reconstruct a [`Status`] projection by folding its events.
//!
//! Statuses differ from projects and tasks: a fixed set of *seed* statuses
//! always exists without anything written to disk. A seed is expressed as a
//! synthetic snapshot event that the [`storage`](crate::storage) layer overlays
//! at read time, so reconstructing a status is the same uniform blind fold as
//! any other entity — by the time [`replay`] runs, the seed snapshot is simply
//! the first event in the stream.
//!
//! This module only declares *which* seeds statuses have (via [`Seeded`]); the
//! generic engine and the read-time overlay both live in
//! [`storage::seeds`](crate::storage::seeds).

use crate::events::status::{StatusEvent, StatusEventKind};
use crate::projections::status::{Status, StatusId, StatusKind};
use crate::storage::seeds::Seeded;

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

/// Statuses own the declaration of their seeds — which slugs exist and the
/// snapshot each one sets. The seeding *engine* and overlay live in
/// [`storage::seeds`](crate::storage::seeds).
impl Seeded for Status {
    type Id = StatusId;
    type EventKind = StatusEventKind;

    const COLLECTION: &'static str = crate::events::status::COLLECTION;

    fn seed_slugs() -> &'static [&'static str] {
        SEED_SLUGS
    }

    fn seed_snapshot(slug: &str) -> StatusEventKind {
        let (name, kind) = match slug {
            "backlog" => ("Backlog", StatusKind::Unstarted),
            "todo" => ("Todo", StatusKind::Unstarted),
            "in_progress" => ("In Progress", StatusKind::Started),
            "in_review" => ("In Review", StatusKind::Started),
            "complete" => ("Complete", StatusKind::Complete),
            "canceled" => ("Canceled", StatusKind::Canceled),
            other => unreachable!("unknown status seed slug: {other}"),
        };
        StatusEventKind::Snapshot {
            name: name.to_string(),
            kind,
            description: None,
            removed: false,
        }
    }
}

/// Rebuild a status by folding its events in chronological order.
///
/// For seed statuses the stream begins with the seed's synthetic snapshot event
/// (see [`storage::seeds::seed_event`](crate::storage::seeds::seed_event));
/// user-created statuses begin with a
/// `Created` (or `Snapshot`) event. Returns `None` if the history establishes
/// no state.
pub fn replay(id: StatusId, events: &[StatusEvent]) -> Option<Status> {
    let mut status: Option<Status> = None;

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
            StatusEventKind::Snapshot { name, kind, description, removed } => {
                // Preserve the original creation time if the status already
                // exists; otherwise this snapshot bootstraps it (a seed's v5
                // snapshot carries no timestamp, so seeds stay `None`).
                let created_at_millis = status
                    .as_ref()
                    .and_then(|s| s.created_at_millis)
                    .or_else(|| event.created_at_millis());
                status = Some(Status {
                    id,
                    name: name.clone(),
                    kind: kind.clone(),
                    description: description.clone(),
                    removed: *removed,
                    created_at_millis,
                });
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::seeds::{seed_event, seed_id_for};

    /// The full event stream for a seed status: its synthetic snapshot first,
    /// then any real events — mirroring what `storage::seeds::load_stream`
    /// assembles. Here we build it by hand to unit-test the fold in isolation.
    fn seed_stream(slug: &str, rest: Vec<StatusEvent>) -> (StatusId, Vec<StatusEvent>) {
        let id = seed_id_for::<Status>(slug);
        let mut events = vec![seed_event::<Status>(slug)];
        events.extend(rest);
        (id, events)
    }

    #[test]
    fn seed_snapshot_alone_reconstructs_the_default() {
        let (id, events) = seed_stream("backlog", vec![]);
        let status = replay(id, &events).unwrap();
        assert_eq!(status.name, "Backlog");
        assert_eq!(status.kind, StatusKind::Unstarted);
        assert!(!status.removed);
        // v5 seed id => no creation time.
        assert!(status.created_at_millis.is_none());
    }

    #[test]
    fn events_mutate_the_seed_default() {
        let (id, events) = seed_stream(
            "backlog",
            vec![
                StatusEvent::new(StatusEventKind::Renamed { new_name: "Ice Box".into() }),
                StatusEvent::new(StatusEventKind::Removed),
            ],
        );
        let status = replay(id, &events).unwrap();
        assert_eq!(status.name, "Ice Box");
        assert!(status.removed);
    }

    #[test]
    fn user_created_needs_no_seed() {
        let id = StatusId::new();
        let events = vec![StatusEvent::new(StatusEventKind::Created {
            name: "Needs QA".into(),
            kind: StatusKind::Started,
            description: Some("awaiting review".into()),
        })];
        let status = replay(id, &events).unwrap();
        assert_eq!(status.name, "Needs QA");
        assert_eq!(status.kind, StatusKind::Started);
        assert!(status.created_at_millis.is_some());
    }

    #[test]
    fn snapshot_overwrites_but_keeps_creation_time() {
        let id = StatusId::new();
        let events = vec![
            StatusEvent::new(StatusEventKind::Created {
                name: "Original".into(),
                kind: StatusKind::Unstarted,
                description: None,
            }),
            StatusEvent::new(StatusEventKind::Snapshot {
                name: "Rewritten".into(),
                kind: StatusKind::Complete,
                description: Some("set wholesale".into()),
                removed: true,
            }),
        ];
        let status = replay(id, &events).unwrap();
        assert_eq!(status.name, "Rewritten");
        assert_eq!(status.kind, StatusKind::Complete);
        assert_eq!(status.description.as_deref(), Some("set wholesale"));
        assert!(status.removed);
        // creation time comes from the original Created event, not the snapshot.
        assert!(status.created_at_millis.is_some());
    }

    #[test]
    fn empty_history_is_none() {
        assert!(replay(StatusId::new(), &[]).is_none());
    }
}
