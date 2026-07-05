//! `tasks task …` — thin adapters over the daemon's `/tasks` routes.
//!
//! The list view resolves status and project IDs to names (two extra local
//! requests) so exploring reads like the app, not like raw storage.

use std::collections::HashMap;

use clap::Subcommand;
use serde_json::{json, Value};

use tasks_core::{Project, Status, Task};

use crate::client::Result;
use crate::cmd::Ctx;
use crate::render;

#[derive(Subcommand)]
pub enum Cmd {
    /// List tasks, oldest first.
    List {
        /// Restrict to one project.
        #[arg(long, value_name = "PROJECT_ID")]
        project: Option<String>,
        /// List closed tasks instead of open ones.
        #[arg(long)]
        closed: bool,
    },
    /// Show one task.
    Show { id: String },
    /// Show a task's full event history, oldest first.
    History { id: String },
    /// Create a task in a project.
    Create {
        name: String,
        #[arg(long, value_name = "PROJECT_ID")]
        project: String,
    },
    /// Rename a task.
    Rename { id: String, new_name: String },
    /// Move a task to a different project.
    Move {
        id: String,
        #[arg(value_name = "PROJECT_ID")]
        project: String,
    },
    /// Close a task.
    Close { id: String },
    /// Reopen a closed task.
    Reopen { id: String },
    /// Set a task's status, or clear it with --clear.
    Status {
        id: String,
        #[arg(value_name = "STATUS_ID", required_unless_present = "clear")]
        status: Option<String>,
        /// Clear the status instead of setting one.
        #[arg(long, conflicts_with = "status")]
        clear: bool,
    },
    /// Replace a task's description.
    Describe { id: String, description: String },
}

pub fn run(ctx: &Ctx, cmd: Cmd) -> Result<()> {
    let ws = ctx.ws()?;
    let scoped: &[(&str, &str)] = &[("workspace", &ws)];
    let value = match &cmd {
        Cmd::List { project, closed } => {
            let mut query = vec![("workspace", ws.as_str())];
            if let Some(project) = project {
                query.push(("project_id", project));
            }
            if *closed {
                query.push(("closed", "true"));
            }
            ctx.daemon.get("/tasks", &query)?
        }
        Cmd::Show { id } => ctx.daemon.get(&format!("/tasks/{id}"), scoped)?,
        Cmd::History { id } => ctx.daemon.get(&format!("/tasks/{id}/events"), scoped)?,
        Cmd::Create { name, project } => ctx.daemon.post(
            "/tasks",
            scoped,
            Some(json!({ "project_id": project, "name": name })),
        )?,
        Cmd::Rename { id, new_name } => ctx.daemon.post(
            &format!("/tasks/{id}/rename"),
            scoped,
            Some(json!({ "new_name": new_name })),
        )?,
        Cmd::Move { id, project } => ctx.daemon.post(
            &format!("/tasks/{id}/move"),
            scoped,
            Some(json!({ "project_id": project })),
        )?,
        Cmd::Close { id } => ctx.daemon.post(&format!("/tasks/{id}/close"), scoped, None)?,
        Cmd::Reopen { id } => ctx
            .daemon
            .post(&format!("/tasks/{id}/reopen"), scoped, None)?,
        Cmd::Status { id, status, .. } => ctx.daemon.post(
            &format!("/tasks/{id}/status"),
            scoped,
            Some(json!({ "status_id": status })),
        )?,
        Cmd::Describe { id, description } => ctx.daemon.post(
            &format!("/tasks/{id}/description"),
            scoped,
            Some(json!({ "description": description })),
        )?,
    };
    if ctx.json {
        return ctx.emit(&value);
    }
    match cmd {
        Cmd::List { .. } => list(ctx, &ws, value),
        Cmd::History { .. } => history(value),
        _ => show(value),
    }
}

/// Render a task list with status and project IDs resolved to names.
fn list(ctx: &Ctx, ws: &str, value: Value) -> Result<()> {
    let tasks: Vec<Task> = serde_json::from_value(value)?;
    let scoped: &[(&str, &str)] = &[("workspace", ws)];
    let statuses: Vec<Status> = serde_json::from_value(ctx.daemon.get("/statuses", scoped)?)?;
    let projects: Vec<Project> = serde_json::from_value(ctx.daemon.get("/projects", scoped)?)?;
    let status_names: HashMap<String, String> = statuses
        .into_iter()
        .map(|s| (s.id.to_string(), s.name))
        .collect();
    let project_names: HashMap<String, String> = projects
        .into_iter()
        .map(|p| (p.id.to_string(), p.name))
        .collect();
    let named = |names: &HashMap<String, String>, id: String| {
        names.get(&id).cloned().unwrap_or(id)
    };
    let rows = tasks
        .into_iter()
        .map(|t| {
            vec![
                t.id.to_string(),
                t.name,
                render::opt(t.status_id.map(|s| named(&status_names, s.to_string()))),
                named(&project_names, t.project_id.to_string()),
                render::time(t.created_at_millis),
            ]
        })
        .collect();
    render::table(&["ID", "NAME", "STATUS", "PROJECT", "CREATED"], rows);
    Ok(())
}

/// Render one task as a key-value block.
fn show(value: Value) -> Result<()> {
    let task: Task = serde_json::from_value(value)?;
    render::kv(&[
        ("id", task.id.to_string()),
        ("name", task.name),
        ("project", task.project_id.to_string()),
        ("status", render::opt(task.status_id.map(|s| s.to_string()))),
        ("closed", task.closed.to_string()),
        ("created", render::time(task.created_at_millis)),
        ("description", render::opt(Some(task.description))),
    ]);
    Ok(())
}

/// Render an event history as `TIME  EVENT  DETAILS` rows.
fn history(value: Value) -> Result<()> {
    let events = value.as_array().cloned().unwrap_or_default();
    let rows = events
        .iter()
        .map(|event| {
            vec![
                render::time(event["created_at_millis"].as_u64()),
                event["type"].as_str().unwrap_or("?").to_string(),
                render::payload(event.get("payload")),
            ]
        })
        .collect();
    render::table(&["TIME", "EVENT", "DETAILS"], rows);
    Ok(())
}
