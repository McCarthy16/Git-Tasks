//! The tasks daemon.
//!
//! A long-running process that owns the application's core logic (via
//! [`tasks_core`]) and its async runtime. Clients — the Tauri app today, a CLI
//! later — drive it over a local HTTP interface rather than embedding the core
//! in-process, so the runtime is decoupled from any single front-end shell.
//!
//! The daemon binds to loopback only: nothing is exposed to the network, which
//! preserves the project's offline / local-first guarantee.

mod http;
mod state;

use std::net::{Ipv4Addr, SocketAddr};

use state::AppState;

/// The address the daemon listens on. Loopback only, by design.
const DEFAULT_ADDR: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let state = AppState::new();
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(DEFAULT_ADDR).await?;
    tracing::info!(addr = %DEFAULT_ADDR, "tasks daemon listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("tasks daemon stopped");
    Ok(())
}

/// Initialize tracing, honoring `RUST_LOG` and defaulting to `info`.
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).init();
}

/// Resolves when the process receives a shutdown signal (Ctrl-C).
async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "failed to install Ctrl-C handler");
    }
    tracing::info!("shutdown signal received");
}
