//! The `Project` projection: a project as rebuilt from its event history.

use serde::{Deserialize, Serialize};

use crate::shared::id::prefixed_id;

prefixed_id!(
    /// Identifier for a project (`project_<hex>`).
    ProjectId,
    "project_"
);

/// A project, fully reconstructed from its events.
///
/// This is pure read-side data — the shape of a project after all its events
/// have been folded. The folding itself lives in the `reconstruction` layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub closed: bool,
    /// Creation time (ms since the Unix epoch), decoded from the `created` event.
    pub created_at_millis: Option<u64>,
}
