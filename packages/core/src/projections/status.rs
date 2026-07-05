//! The `Status` projection: a workflow status as rebuilt from its event history.

use serde::{Deserialize, Serialize};

use crate::shared::id::prefixed_id;

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

/// A status, fully reconstructed from its events (or as a seed default).
///
/// This is pure read-side data — the shape of a status after all its events
/// have been folded. The folding itself lives in the `reconstruction` layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub id: StatusId,
    pub name: String,
    pub kind: StatusKind,
    pub description: Option<String>,
    pub removed: bool,
    /// `None` for seed statuses that have never been written to disk.
    pub created_at_millis: Option<u64>,
}
