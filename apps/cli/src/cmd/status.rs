//! `tasks status …` — thin adapters over the daemon's `/statuses` routes.

use clap::Subcommand;
use serde_json::json;

use tasks_core::{Status, StatusKind};

use crate::client::Result;
use crate::cmd::Ctx;
use crate::render;

#[derive(Subcommand)]
pub enum Cmd {
    /// List statuses in canonical (seeds-first) order.
    List {
        /// List soft-removed statuses instead of active ones.
        #[arg(long)]
        removed: bool,
    },
    /// Create a status.
    Create {
        name: String,
        #[arg(long, value_enum)]
        kind: Kind,
        #[arg(long)]
        description: Option<String>,
    },
    /// Rename a status.
    Rename { id: String, new_name: String },
    /// Update a status's description, or clear it with --clear.
    Describe {
        id: String,
        #[arg(required_unless_present = "clear")]
        description: Option<String>,
        /// Clear the description instead of setting one.
        #[arg(long, conflicts_with = "description")]
        clear: bool,
    },
    /// Change a status's semantic kind.
    Kind {
        id: String,
        #[arg(value_enum)]
        kind: Kind,
    },
    /// Soft-remove a status.
    Remove { id: String },
}

/// [`StatusKind`] as a CLI argument.
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Kind {
    Unstarted,
    Started,
    Complete,
    Canceled,
}

impl Kind {
    /// The wire name (matches `StatusKind`'s snake_case serialization).
    fn as_str(self) -> &'static str {
        match self {
            Kind::Unstarted => "unstarted",
            Kind::Started => "started",
            Kind::Complete => "complete",
            Kind::Canceled => "canceled",
        }
    }
}

/// [`StatusKind`] back to its wire name, for display.
fn kind_str(kind: &StatusKind) -> &'static str {
    match kind {
        StatusKind::Unstarted => "unstarted",
        StatusKind::Started => "started",
        StatusKind::Complete => "complete",
        StatusKind::Canceled => "canceled",
    }
}

pub fn run(ctx: &Ctx, cmd: Cmd) -> Result<()> {
    let ws = ctx.ws()?;
    let scoped: &[(&str, &str)] = &[("workspace", &ws)];
    let value = match &cmd {
        Cmd::List { removed } => {
            let mut query = vec![("workspace", ws.as_str())];
            if *removed {
                query.push(("removed", "true"));
            }
            ctx.daemon.get("/statuses", &query)?
        }
        Cmd::Create {
            name,
            kind,
            description,
        } => ctx.daemon.post(
            "/statuses",
            scoped,
            Some(json!({ "name": name, "kind": kind.as_str(), "description": description })),
        )?,
        Cmd::Rename { id, new_name } => ctx.daemon.post(
            &format!("/statuses/{id}/rename"),
            scoped,
            Some(json!({ "new_name": new_name })),
        )?,
        Cmd::Describe {
            id, description, ..
        } => ctx.daemon.post(
            &format!("/statuses/{id}/description"),
            scoped,
            Some(json!({ "description": description })),
        )?,
        Cmd::Kind { id, kind } => ctx.daemon.post(
            &format!("/statuses/{id}/kind"),
            scoped,
            Some(json!({ "new_kind": kind.as_str() })),
        )?,
        Cmd::Remove { id } => ctx
            .daemon
            .post(&format!("/statuses/{id}/remove"), scoped, None)?,
    };
    if ctx.json {
        return ctx.emit(&value);
    }
    match cmd {
        Cmd::List { .. } => {
            let statuses: Vec<Status> = serde_json::from_value(value)?;
            let rows = statuses
                .into_iter()
                .map(|s| {
                    vec![
                        s.id.to_string(),
                        s.name,
                        kind_str(&s.kind).to_string(),
                        render::opt(s.description),
                    ]
                })
                .collect();
            render::table(&["ID", "NAME", "KIND", "DESCRIPTION"], rows);
        }
        _ => {
            let status: Status = serde_json::from_value(value)?;
            render::kv(&[
                ("id", status.id.to_string()),
                ("name", status.name),
                ("kind", kind_str(&status.kind).to_string()),
                ("description", render::opt(status.description)),
                ("removed", status.removed.to_string()),
            ]);
        }
    }
    Ok(())
}
