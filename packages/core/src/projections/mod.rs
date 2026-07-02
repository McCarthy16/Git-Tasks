//! Projections — the fully reconstructed objects.
//!
//! The read-side models built by folding events during reconstruction. These
//! are pure data: each entity owns its typed ID and its reconstructed shape,
//! but the folding logic that produces them lives in the `reconstruction`
//! layer, not here.
//!
//! Three entities are modeled today:
//!
//! - [`project::Project`] — a project.
//! - [`task::Task`] — a task, which always belongs to a project.
//! - [`status::Status`] — a workflow status.

pub mod project;
pub mod status;
pub mod task;

pub use project::{Project, ProjectId};
pub use status::{Status, StatusId, StatusKind};
pub use task::{Task, TaskId};
