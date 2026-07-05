//! The `Task` projection: a task as rebuilt from its event history.

use serde::{Deserialize, Serialize};

use crate::projections::project::ProjectId;
use crate::projections::status::StatusId;
use crate::shared::id::prefixed_id;

prefixed_id!(
    /// Identifier for a task (`task_<hex>`).
    TaskId,
    "task_"
);

/// A task, fully reconstructed from its events.
///
/// This is pure read-side data — the shape of a task after all its events have
/// been folded. The folding itself lives in the `reconstruction` layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    /// The project this task currently belongs to (may change via `moved`).
    pub project_id: ProjectId,
    pub name: String,
    pub description: String,
    /// `None` means the task has no status assigned.
    pub status_id: Option<StatusId>,
    pub closed: bool,
    /// Creation time (ms since the Unix epoch), decoded from the `created` event.
    pub created_at_millis: Option<u64>,
}
