//! Task events — the event code for the task entity.

use serde::{Deserialize, Serialize};

use crate::events::envelope::{Event, EventKind};
use crate::projections::project::ProjectId;
use crate::projections::status::StatusId;

/// The event-store collection holding task event streams.
pub const COLLECTION: &str = "tasks";

/// An event in a task's history.
///
/// The task↔project relationship lives in the event payload (not the folder
/// tree), so `Moved` can change the owning project without touching file layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum TaskEventKind {
    Created { project_id: ProjectId, name: String },
    Renamed { new_name: String },
    Moved { new_project_id: ProjectId },
    Closed,
    Reopened,
    DescriptionUpdated { description: String },
    /// `None` means "no status" (explicitly cleared).
    StatusChanged { status_id: Option<StatusId> },
    /// Sets the entire task state at once — every mutable attribute.
    /// Bootstraps the task if it's the first event; otherwise overwrites.
    Snapshot {
        project_id: ProjectId,
        name: String,
        description: String,
        status_id: Option<StatusId>,
        closed: bool,
    },
}

impl EventKind for TaskEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            TaskEventKind::Created { .. } => "created",
            TaskEventKind::Renamed { .. } => "renamed",
            TaskEventKind::Moved { .. } => "moved",
            TaskEventKind::Closed => "closed",
            TaskEventKind::Reopened => "reopened",
            TaskEventKind::DescriptionUpdated { .. } => "description_updated",
            TaskEventKind::StatusChanged { .. } => "status_changed",
            TaskEventKind::Snapshot { .. } => "snapshot",
        }
    }
}

/// A task event file.
pub type TaskEvent = Event<TaskEventKind>;
