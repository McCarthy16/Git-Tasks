//! Reconstruct a [`Project`] projection by folding its events.

use crate::events::project::{ProjectEvent, ProjectEventKind};
use crate::projections::project::{Project, ProjectId};

/// Rebuild a project by folding its events in chronological order.
///
/// Returns `None` if the history does not start with a `created` event.
pub fn replay(id: ProjectId, events: &[ProjectEvent]) -> Option<Project> {
    let mut project: Option<Project> = None;

    for event in events {
        match &event.kind {
            ProjectEventKind::Created { name } => {
                project = Some(Project {
                    id,
                    name: name.clone(),
                    closed: false,
                    created_at_millis: event.created_at_millis(),
                });
            }
            ProjectEventKind::Renamed { new_name } => {
                if let Some(p) = project.as_mut() {
                    p.name = new_name.clone();
                }
            }
            ProjectEventKind::Closed => {
                if let Some(p) = project.as_mut() {
                    p.closed = true;
                }
            }
            ProjectEventKind::Reopened => {
                if let Some(p) = project.as_mut() {
                    p.closed = false;
                }
            }
            ProjectEventKind::Snapshot { name, closed } => {
                // Preserve the original creation time if the project already
                // exists; otherwise this snapshot bootstraps it.
                let created_at_millis = project
                    .as_ref()
                    .and_then(|p| p.created_at_millis)
                    .or_else(|| event.created_at_millis());
                project = Some(Project {
                    id,
                    name: name.clone(),
                    closed: *closed,
                    created_at_millis,
                });
            }
        }
    }

    project
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::project::ProjectEvent;

    #[test]
    fn folds_created_then_renamed() {
        let id = ProjectId::new();
        let events = vec![
            ProjectEvent::new(ProjectEventKind::Created { name: "Original".into() }),
            ProjectEvent::new(ProjectEventKind::Renamed { new_name: "Updated".into() }),
        ];

        let project = replay(id, &events).unwrap();
        assert_eq!(project.id, id);
        assert_eq!(project.name, "Updated");
        assert!(!project.closed);
        assert!(project.created_at_millis.is_some());
    }

    #[test]
    fn close_then_reopen_toggles_flag() {
        let id = ProjectId::new();
        let events = vec![
            ProjectEvent::new(ProjectEventKind::Created { name: "P".into() }),
            ProjectEvent::new(ProjectEventKind::Closed),
            ProjectEvent::new(ProjectEventKind::Reopened),
        ];
        assert!(!replay(id, &events).unwrap().closed);
    }

    #[test]
    fn history_without_created_is_none() {
        let id = ProjectId::new();
        let events = vec![ProjectEvent::new(ProjectEventKind::Renamed {
            new_name: "orphan".into(),
        })];
        assert!(replay(id, &events).is_none());
    }

    #[test]
    fn snapshot_bootstraps_then_later_snapshot_keeps_creation_time() {
        let id = ProjectId::new();
        let boot = replay(
            id,
            &[ProjectEvent::new(ProjectEventKind::Snapshot {
                name: "Booted".into(),
                closed: true,
            })],
        )
        .unwrap();
        assert_eq!(boot.name, "Booted");
        assert!(boot.closed);
        assert!(boot.created_at_millis.is_some());

        let events = vec![
            ProjectEvent::new(ProjectEventKind::Created { name: "Original".into() }),
            ProjectEvent::new(ProjectEventKind::Snapshot { name: "Reset".into(), closed: true }),
        ];
        let project = replay(id, &events).unwrap();
        assert_eq!(project.name, "Reset");
        assert!(project.closed);
        assert!(project.created_at_millis.is_some());
    }
}
