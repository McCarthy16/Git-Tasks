//! Status events — the event code for the status entity.

use serde::{Deserialize, Serialize};

use crate::events::envelope::{Event, EventKind};
use crate::projections::status::StatusKind;

/// The event-store collection holding status event streams.
pub const COLLECTION: &str = "statuses";

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
    /// Sets the entire status state at once — every mutable attribute.
    /// Bootstraps the status if it's the first event; otherwise overwrites.
    /// Seed statuses are expressed as a snapshot (see the reconstruction layer).
    Snapshot {
        name: String,
        kind: StatusKind,
        description: Option<String>,
        removed: bool,
    },
}

impl EventKind for StatusEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            StatusEventKind::Created { .. } => "created",
            StatusEventKind::Renamed { .. } => "renamed",
            StatusEventKind::DescriptionUpdated { .. } => "description_updated",
            StatusEventKind::KindChanged { .. } => "kind_changed",
            StatusEventKind::Removed => "removed",
            StatusEventKind::Snapshot { .. } => "snapshot",
        }
    }
}

/// A status event file.
pub type StatusEvent = Event<StatusEventKind>;
