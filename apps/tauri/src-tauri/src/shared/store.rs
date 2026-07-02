//! Filesystem helpers for the append-only event store.
//!
//! A [`Workspace`] is a chosen repository folder and its `.tasks` event store.
//! The free functions ([`append`], [`read_all`], [`list_ids`]) are the generic,
//! domain-agnostic primitives the `projects` and `tasks` domains call to persist
//! and rebuild their aggregates.
//!
//! Layout, rooted at the workspace:
//!
//! ```text
//! .tasks/
//!   projects/project_<hex>/events/<event-hex>-<type>.json
//!   tasks/task_<hex>/events/<event-hex>-<type>.json
//! ```
//!
//! Events are never mutated. Each is written once to its own uniquely-named
//! file (UUIDv7 hex prefix → unique + chronological), so concurrent
//! contributors don't collide.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::Result;
use crate::shared::event::{Event, EventKind};

/// A chosen repository folder and the `.tasks` event store inside it.
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Open a workspace rooted at `root`. Nothing is created until the first
    /// event is appended (or [`ensure`](Self::ensure) is called).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The chosen working folder.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The `.tasks` directory holding all event data.
    pub fn dot_tasks(&self) -> PathBuf {
        self.root.join(".tasks")
    }

    /// The directory holding a collection's entities, e.g. `.tasks/projects`.
    pub fn collection_dir(&self, collection: &str) -> PathBuf {
        self.dot_tasks().join(collection)
    }

    /// The events directory for one entity, e.g.
    /// `.tasks/projects/project_<hex>/events`.
    pub fn events_dir(&self, collection: &str, id: impl Display) -> PathBuf {
        self.collection_dir(collection)
            .join(id.to_string())
            .join("events")
    }

    /// Find-or-create the `.tasks` tree for the given collections. Idempotent —
    /// safe to call on every launch whether or not the folders already exist.
    pub fn ensure(&self, collections: &[&str]) -> Result<()> {
        for collection in collections {
            fs::create_dir_all(self.collection_dir(collection))?;
        }
        Ok(())
    }
}

/// Append one event to `events_dir`, creating the directory if needed.
///
/// The filename is `<event-hex>-<type>.json` — the UUIDv7 hex keeps writes
/// unique and chronologically ordered, while the type suffix makes the file
/// self-describing.
pub fn append<K: Serialize + EventKind>(events_dir: &Path, event: &Event<K>) -> Result<()> {
    fs::create_dir_all(events_dir)?;
    let path = events_dir.join(format!("{}-{}.json", event.id, event.kind.event_type()));
    fs::write(path, serde_json::to_string_pretty(event)?)?;
    Ok(())
}

/// Read and parse every `*.json` event in `events_dir`, sorted by filename
/// (chronological). A missing directory yields an empty vec.
pub fn read_all<K: DeserializeOwned>(events_dir: &Path) -> Result<Vec<Event<K>>> {
    let mut files = match fs::read_dir(events_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    files.sort();

    let mut events = Vec::with_capacity(files.len());
    for path in files {
        events.push(serde_json::from_str(&fs::read_to_string(path)?)?);
    }
    Ok(events)
}

/// List entity IDs by reading the immediate subdirectory names of `dir` and
/// parsing each as `Id`. Names that don't parse are skipped. A missing
/// directory yields an empty vec. Sorted oldest-first.
pub fn list_ids<Id: FromStr + Ord>(dir: &Path) -> Result<Vec<Id>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut ids = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            if let Ok(id) = name.parse::<Id>() {
                ids.push(id);
            }
        }
    }
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_finds_or_creates_collections() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace::new(tmp.path());

        assert!(!ws.dot_tasks().exists());
        ws.ensure(&["projects", "tasks"]).unwrap();
        assert!(ws.collection_dir("projects").is_dir());
        assert!(ws.collection_dir("tasks").is_dir());

        // Idempotent.
        ws.ensure(&["projects", "tasks"]).unwrap();
        assert!(ws.collection_dir("projects").is_dir());
    }
}
