//! The tasks CLI — a terminal client for the tasks daemon.
//!
//! Explores and edits `.tasks` workspaces the same way the Tauri app does:
//! every command maps onto one daemon route, so no core logic lives here.
//! The workspace defaults to the nearest ancestor directory containing
//! `.tasks` (like git's repo discovery) and can be overridden per invocation
//! with `--workspace`. `--json` swaps the human-readable rendering for the
//! daemon's raw responses, for scripting.

mod client;
mod cmd;
mod render;
mod workspace;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::client::Daemon;
use crate::cmd::Ctx;

#[derive(Parser)]
#[command(
    name = "tasks",
    version,
    about = "Explore and edit tasks workspaces from the terminal, via the tasks daemon"
)]
struct Cli {
    /// Workspace root. Defaults to the nearest ancestor directory containing `.tasks`.
    #[arg(long, global = true, value_name = "PATH")]
    workspace: Option<PathBuf>,

    /// Base URL of the tasks daemon.
    #[arg(
        long,
        global = true,
        env = "TASKS_DAEMON",
        default_value = "http://127.0.0.1:4000",
        value_name = "URL"
    )]
    daemon: String,

    /// Print raw JSON responses instead of human-readable output.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a `.tasks` workspace in a directory (defaults to the current one).
    Init {
        /// Directory to initialize (must already exist).
        path: Option<PathBuf>,
    },
    /// Check that the daemon is reachable.
    Health,
    /// Explore and edit projects.
    #[command(subcommand)]
    Project(cmd::project::Cmd),
    /// Explore and edit tasks.
    #[command(subcommand)]
    Task(cmd::task::Cmd),
    /// Explore and edit workflow statuses.
    #[command(subcommand)]
    Status(cmd::status::Cmd),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = Ctx {
        daemon: Daemon::new(&cli.daemon),
        workspace: cli.workspace,
        json: cli.json,
    };
    let result = match cli.command {
        Command::Init { path } => cmd::workspace::init(&ctx, path),
        Command::Health => cmd::workspace::health(&ctx),
        Command::Project(command) => cmd::project::run(&ctx, command),
        Command::Task(command) => cmd::task::run(&ctx, command),
        Command::Status(command) => cmd::status::run(&ctx, command),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
