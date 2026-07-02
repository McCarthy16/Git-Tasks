//! Reconstruct a [`Task`] projection by folding its events.

use crate::events::task::{TaskEvent, TaskEventKind};
use crate::projections::task::{Task, TaskId};

/// Rebuild a task by folding its events in chronological order.
///
/// Returns `None` if the history does not start with a `created` event.
pub fn replay(id: TaskId, events: &[TaskEvent]) -> Option<Task> {
    let mut task: Option<Task> = None;

    for event in events {
        match &event.kind {
            TaskEventKind::Created { project_id, name } => {
                task = Some(Task {
                    id,
                    project_id: *project_id,
                    name: name.clone(),
                    description: String::new(),
                    status_id: None,
                    closed: false,
                    created_at_millis: event.created_at_millis(),
                });
            }
            TaskEventKind::Renamed { new_name } => {
                if let Some(t) = task.as_mut() {
                    t.name = new_name.clone();
                }
            }
            TaskEventKind::Moved { new_project_id } => {
                if let Some(t) = task.as_mut() {
                    t.project_id = *new_project_id;
                }
            }
            TaskEventKind::Closed => {
                if let Some(t) = task.as_mut() {
                    t.closed = true;
                }
            }
            TaskEventKind::Reopened => {
                if let Some(t) = task.as_mut() {
                    t.closed = false;
                }
            }
            TaskEventKind::DescriptionUpdated { description } => {
                if let Some(t) = task.as_mut() {
                    t.description = description.clone();
                }
            }
            TaskEventKind::StatusChanged { status_id } => {
                if let Some(t) = task.as_mut() {
                    t.status_id = *status_id;
                }
            }
            TaskEventKind::Snapshot {
                project_id,
                name,
                description,
                status_id,
                closed,
            } => {
                // Preserve the original creation time if the task already
                // exists; otherwise this snapshot bootstraps it.
                let created_at_millis = task
                    .as_ref()
                    .and_then(|t| t.created_at_millis)
                    .or_else(|| event.created_at_millis());
                task = Some(Task {
                    id,
                    project_id: *project_id,
                    name: name.clone(),
                    description: description.clone(),
                    status_id: *status_id,
                    closed: *closed,
                    created_at_millis,
                });
            }
        }
    }

    task
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projections::project::ProjectId;
    use crate::projections::status::StatusId;

    #[test]
    fn folds_a_full_history() {
        let id = TaskId::new();
        let project_a = ProjectId::new();
        let project_b = ProjectId::new();
        let status = StatusId::new();

        let events = vec![
            TaskEvent::new(TaskEventKind::Created {
                project_id: project_a,
                name: "First".into(),
            }),
            TaskEvent::new(TaskEventKind::Renamed { new_name: "Second".into() }),
            TaskEvent::new(TaskEventKind::DescriptionUpdated { description: "notes".into() }),
            TaskEvent::new(TaskEventKind::Moved { new_project_id: project_b }),
            TaskEvent::new(TaskEventKind::StatusChanged { status_id: Some(status) }),
            TaskEvent::new(TaskEventKind::Closed),
        ];

        let task = replay(id, &events).unwrap();
        assert_eq!(task.id, id);
        assert_eq!(task.name, "Second");
        assert_eq!(task.description, "notes");
        assert_eq!(task.project_id, project_b);
        assert_eq!(task.status_id, Some(status));
        assert!(task.closed);
    }

    #[test]
    fn status_can_be_cleared() {
        let id = TaskId::new();
        let events = vec![
            TaskEvent::new(TaskEventKind::Created {
                project_id: ProjectId::new(),
                name: "T".into(),
            }),
            TaskEvent::new(TaskEventKind::StatusChanged { status_id: Some(StatusId::new()) }),
            TaskEvent::new(TaskEventKind::StatusChanged { status_id: None }),
        ];
        assert!(replay(id, &events).unwrap().status_id.is_none());
    }

    #[test]
    fn history_without_created_is_none() {
        let id = TaskId::new();
        let events = vec![TaskEvent::new(TaskEventKind::Closed)];
        assert!(replay(id, &events).is_none());
    }

    #[test]
    fn snapshot_sets_every_attribute_at_once() {
        let id = TaskId::new();
        let project = ProjectId::new();
        let status = StatusId::new();
        let events = vec![TaskEvent::new(TaskEventKind::Snapshot {
            project_id: project,
            name: "Wholesale".into(),
            description: "all at once".into(),
            status_id: Some(status),
            closed: true,
        })];
        let task = replay(id, &events).unwrap();
        assert_eq!(task.project_id, project);
        assert_eq!(task.name, "Wholesale");
        assert_eq!(task.description, "all at once");
        assert_eq!(task.status_id, Some(status));
        assert!(task.closed);
        assert!(task.created_at_millis.is_some());
    }
}
