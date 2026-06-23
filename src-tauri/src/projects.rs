//! The projects domain: the `Project` aggregate, its events, and the helpers
//! to create, load, and list projects in a workspace.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::shared::event::{Event, EventKind};
use crate::shared::id::prefixed_id;
use crate::shared::store::{self, Workspace};

/// The `.tasks` subfolder holding project event streams.
pub const COLLECTION: &str = "projects";

prefixed_id!(
    /// Identifier for a project (`project_<hex>`).
    ProjectId,
    "project_"
);

/// An event in a project's history.
///
/// Adjacently tagged so it serializes as `{ "type": ..., "payload": ... }`
/// inside the [`Event`] envelope. Only `created` exists for now; future
/// variants (`renamed`, `closed`, ...) are added here and to the [`EventKind`]
/// impl below.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ProjectEventKind {
    Created { name: String },
}

impl EventKind for ProjectEventKind {
    fn event_type(&self) -> &'static str {
        match self {
            ProjectEventKind::Created { .. } => "created",
        }
    }
}

/// A project event file.
pub type ProjectEvent = Event<ProjectEventKind>;

/// A project, as rebuilt by replaying its events.
#[derive(Clone, Debug, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    /// Creation time (ms since the Unix epoch), decoded from the `created` event.
    pub created_at_millis: Option<u64>,
}

impl Project {
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
                        created_at_millis: event.created_at_millis(),
                    });
                }
            }
        }

        project
    }
}

// --- Helpers: persistence for the projects domain -----------------------

/// Create a project by appending its `created` event, returning the rebuilt
/// project.
pub fn create(ws: &Workspace, name: impl Into<String>) -> Result<Project> {
    let id = ProjectId::new();
    let event = ProjectEvent::new(ProjectEventKind::Created { name: name.into() });
    store::append(&ws.events_dir(COLLECTION, id), &event)?;
    Project::replay(id, std::slice::from_ref(&event)).ok_or(Error::NotCreated)
}

/// Load a single project, or `None` if it has no events on disk.
pub fn load(ws: &Workspace, id: ProjectId) -> Result<Option<Project>> {
    let events: Vec<ProjectEvent> = store::read_all(&ws.events_dir(COLLECTION, id))?;
    if events.is_empty() {
        return Ok(None);
    }
    Ok(Project::replay(id, &events))
}

/// List every project in the workspace, oldest first.
pub fn list(ws: &Workspace) -> Result<Vec<Project>> {
    let mut projects = Vec::new();
    for id in store::list_ids::<ProjectId>(&ws.collection_dir(COLLECTION))? {
        if let Some(project) = load(ws, id)? {
            projects.push(project);
        }
    }
    Ok(projects)
}

/// Whether a project with `id` exists in the workspace.
pub fn exists(ws: &Workspace, id: ProjectId) -> Result<bool> {
    Ok(load(ws, id)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_and_rejects_wrong_prefix() {
        let id = ProjectId::new();
        let text = id.to_string();
        assert!(text.starts_with("project_"));
        assert_eq!(text.parse::<ProjectId>().unwrap(), id);

        let task_id = crate::tasks::TaskId::new().to_string();
        assert!(task_id.parse::<ProjectId>().is_err());
    }

    #[test]
    fn creates_and_reloads_a_project() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        let created = create(&ws, "Roadmap").unwrap();
        assert_eq!(created.name, "Roadmap");
        assert!(created.created_at_millis.is_some());

        // Event file landed in the expected location, named with its type.
        let events_dir = ws.events_dir(COLLECTION, created.id);
        let files: Vec<_> = std::fs::read_dir(&events_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("-created.json"), "got {}", files[0]);

        // Reload from disk via replay.
        let reloaded = load(&ws, created.id).unwrap().unwrap();
        assert_eq!(reloaded.id, created.id);
        assert_eq!(reloaded.name, "Roadmap");
    }

    #[test]
    fn event_round_trips_as_expected_json() {
        // The envelope must flatten to { id, type, payload }.
        let event = ProjectEvent::new(ProjectEventKind::Created {
            name: "Build event store".into(),
        });
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();

        assert_eq!(value["type"], "created");
        assert_eq!(value["payload"]["name"], "Build event store");
        assert!(!value["id"].as_str().unwrap().is_empty());

        let back: ProjectEvent = serde_json::from_value(value).unwrap();
        assert_eq!(back.id, event.id);
    }

    #[test]
    fn missing_project_loads_as_none() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());
        assert!(load(&ws, ProjectId::new()).unwrap().is_none());
        assert!(list(&ws).unwrap().is_empty());
    }
}
