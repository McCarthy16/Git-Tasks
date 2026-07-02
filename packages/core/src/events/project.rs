//! Project events — the event code for the project entity.

use serde::{Deserialize, Serialize};

use crate::events::envelope::{Event, EventKind};

/// The event-store collection holding project event streams.
pub const COLLECTION: &str = "projects";

/// An event in a project's history.
///
/// Adjacently tagged so it serializes as `{ "type": ..., "payload": ... }`
/// inside the [`Event`] envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ProjectEventKind {
    Created { name: String },
    Renamed { new_name: String },
    Closed,
    Reopened,
    /// Sets the entire project state at once — every mutable attribute.
    /// Bootstraps the project if it's the first event; otherwise overwrites.
    Snapshot { name: String, closed: bool },
}

impl EventKind for ProjectEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            ProjectEventKind::Created { .. } => "created",
            ProjectEventKind::Renamed { .. } => "renamed",
            ProjectEventKind::Closed => "closed",
            ProjectEventKind::Reopened => "reopened",
            ProjectEventKind::Snapshot { .. } => "snapshot",
        }
    }
}

/// A project event file.
pub type ProjectEvent = Event<ProjectEventKind>;
