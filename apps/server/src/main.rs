//! The tasks daemon.
//!
//! A long-running process that owns access to the stored state — the
//! event-sourced `.tasks` data inside any workspace — driven through
//! [`tasks_core`] and exposed over a local HTTP interface as plain data
//! operations. It is not tied to a single repository: one daemon serves every
//! workspace on the machine, to any number of clients at once; each request
//! names the workspace it operates on. App logic (navigation, selection,
//! screens) lives in clients like the Tauri app.
//!
//! The daemon binds to loopback only: nothing is exposed to the network, which
//! preserves the project's offline / local-first guarantee.

mod error;
mod http;

use std::net::{Ipv4Addr, SocketAddr};

/// The address the daemon listens on. Loopback only, by design.
const DEFAULT_ADDR: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let router = http::router();

    let listener = tokio::net::TcpListener::bind(DEFAULT_ADDR).await?;
    tracing::info!(addr = %DEFAULT_ADDR, "tasks daemon listening");

    axum::serve(listener, router)
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
