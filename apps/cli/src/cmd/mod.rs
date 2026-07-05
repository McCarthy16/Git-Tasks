//! Command implementations — one module per entity, mirroring the daemon's
//! routes (and, behind them, `tasks_core::commands`).

pub mod project;
pub mod status;
pub mod task;
pub mod workspace;

use std::path::PathBuf;

use serde_json::Value;

use crate::client::{Daemon, Result};

/// Shared context every command runs with.
pub struct Ctx {
    pub daemon: Daemon,
    /// The raw `--workspace` flag; resolved lazily so `init` and `health`
    /// work outside any workspace.
    pub workspace: Option<PathBuf>,
    /// Print raw JSON responses instead of human-readable output.
    pub json: bool,
}

impl Ctx {
    /// The workspace root as the daemon expects it: an absolute path string.
    pub fn ws(&self) -> Result<String> {
        Ok(crate::workspace::resolve(self.workspace.clone())?
            .display()
            .to_string())
    }

    /// Emit a raw JSON response (the `--json` output mode).
    pub fn emit(&self, value: &Value) -> Result<()> {
        println!("{}", serde_json::to_string_pretty(value)?);
        Ok(())
    }
}
