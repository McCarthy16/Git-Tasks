//! `tasks init` / `tasks health` — workspace scaffolding and daemon liveness.

use std::path::PathBuf;

use serde_json::json;

use crate::client::Result;
use crate::cmd::Ctx;

/// Find-or-create the `.tasks` scaffold in `path` (or the current directory).
/// The daemon resolves nothing relative to the caller, so the path is made
/// absolute here before it crosses the wire.
pub fn init(ctx: &Ctx, path: Option<PathBuf>) -> Result<()> {
    let path = match path.or_else(|| ctx.workspace.clone()) {
        Some(path) => std::fs::canonicalize(path)?,
        None => std::env::current_dir()?,
    };
    let value = ctx.daemon.post("/workspaces", &[], Some(json!({ "path": path })))?;
    if ctx.json {
        return ctx.emit(&value);
    }
    println!(
        "initialized workspace at {} ({})",
        value["root"].as_str().unwrap_or("?"),
        value["tasks_dir"].as_str().unwrap_or("?"),
    );
    Ok(())
}

/// Probe the daemon's `/health` route.
pub fn health(ctx: &Ctx) -> Result<()> {
    let value = ctx.daemon.get("/health", &[])?;
    if ctx.json {
        return ctx.emit(&value);
    }
    println!("daemon ok at {}", ctx.daemon.base());
    Ok(())
}
