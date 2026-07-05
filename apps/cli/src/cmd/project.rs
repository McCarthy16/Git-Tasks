//! `tasks project …` — thin adapters over the daemon's `/projects` routes.

use clap::Subcommand;
use serde_json::json;

use tasks_core::Project;

use crate::client::Result;
use crate::cmd::Ctx;
use crate::render;

#[derive(Subcommand)]
pub enum Cmd {
    /// List projects, oldest first.
    List {
        /// List closed (archived) projects instead of open ones.
        #[arg(long)]
        closed: bool,
    },
    /// Show one project.
    Show { id: String },
    /// Create a project.
    Create { name: String },
    /// Rename a project.
    Rename { id: String, new_name: String },
    /// Close (archive) a project.
    Close { id: String },
    /// Reopen a closed project.
    Reopen { id: String },
}

pub fn run(ctx: &Ctx, cmd: Cmd) -> Result<()> {
    let ws = ctx.ws()?;
    let scoped: &[(&str, &str)] = &[("workspace", &ws)];
    let value = match &cmd {
        Cmd::List { closed } => {
            let mut query = vec![("workspace", ws.as_str())];
            if *closed {
                query.push(("closed", "true"));
            }
            ctx.daemon.get("/projects", &query)?
        }
        Cmd::Show { id } => ctx.daemon.get(&format!("/projects/{id}"), scoped)?,
        Cmd::Create { name } => {
            ctx.daemon
                .post("/projects", scoped, Some(json!({ "name": name })))?
        }
        Cmd::Rename { id, new_name } => ctx.daemon.post(
            &format!("/projects/{id}/rename"),
            scoped,
            Some(json!({ "new_name": new_name })),
        )?,
        Cmd::Close { id } => ctx
            .daemon
            .post(&format!("/projects/{id}/close"), scoped, None)?,
        Cmd::Reopen { id } => ctx
            .daemon
            .post(&format!("/projects/{id}/reopen"), scoped, None)?,
    };
    if ctx.json {
        return ctx.emit(&value);
    }
    match cmd {
        Cmd::List { .. } => {
            let projects: Vec<Project> = serde_json::from_value(value)?;
            let rows = projects
                .into_iter()
                .map(|p| vec![p.id.to_string(), p.name, render::time(p.created_at_millis)])
                .collect();
            render::table(&["ID", "NAME", "CREATED"], rows);
        }
        _ => {
            let project: Project = serde_json::from_value(value)?;
            render::kv(&[
                ("id", project.id.to_string()),
                ("name", project.name),
                ("closed", project.closed.to_string()),
                ("created", render::time(project.created_at_millis)),
            ]);
        }
    }
    Ok(())
}
