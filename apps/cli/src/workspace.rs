//! Workspace resolution: which repo root a command operates on.
//!
//! Mirrors git's repo discovery: an explicit `--workspace` wins; otherwise
//! walk up from the current directory until a `.tasks` folder appears.

use std::path::PathBuf;

use crate::client::{Error, Result};

/// Resolve the workspace root a command should operate on, as an absolute path.
pub fn resolve(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(std::fs::canonicalize(path)?);
    }
    let start = std::env::current_dir()?;
    let mut dir = start.clone();
    loop {
        if dir.join(".tasks").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(Error::WorkspaceNotFound(start));
        }
    }
}
