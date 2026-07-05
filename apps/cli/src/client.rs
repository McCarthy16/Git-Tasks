//! The daemon HTTP client.
//!
//! A thin wrapper over `ureq`: every CLI command maps onto one daemon route,
//! exchanging JSON. The daemon's `{ "error": … }` payloads surface as readable
//! messages, and a connection failure points at the likely cause (the daemon
//! isn't running) instead of dumping a transport error.

use std::path::PathBuf;

use serde_json::Value;

/// Errors the CLI reports to the user.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "no `.tasks` workspace found from `{}` upward — pass --workspace <path> or run `tasks init`",
        .0.display()
    )]
    WorkspaceNotFound(PathBuf),
    #[error("{message}")]
    Api { status: u16, message: String },
    #[error("cannot reach the tasks daemon at {base} — is `tasks-server` running?")]
    Unreachable { base: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unexpected response from the daemon: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// A handle on the daemon: its base URL, plus GET/POST helpers.
pub struct Daemon {
    base: String,
}

impl Daemon {
    pub fn new(base: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
        }
    }

    /// The daemon's base URL, for display.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// GET a route, returning the parsed JSON response.
    pub fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let mut request = ureq::get(&format!("{}{path}", self.base));
        for (name, value) in query {
            request = request.query(name, value);
        }
        self.finish(request.call())
    }

    /// POST a route (with an optional JSON body), returning the parsed JSON response.
    pub fn post(&self, path: &str, query: &[(&str, &str)], body: Option<Value>) -> Result<Value> {
        let mut request = ureq::post(&format!("{}{path}", self.base));
        for (name, value) in query {
            request = request.query(name, value);
        }
        let result = match body {
            Some(body) => request.send_json(body),
            None => request.call(),
        };
        self.finish(result)
    }

    /// Fold a `ureq` outcome into ours: parse success bodies, surface the
    /// daemon's error message on HTTP errors, blame a missing daemon otherwise.
    fn finish(&self, result: std::result::Result<ureq::Response, ureq::Error>) -> Result<Value> {
        match result {
            Ok(response) => Ok(response.into_json()?),
            Err(ureq::Error::Status(status, response)) => {
                let message = response
                    .into_json::<Value>()
                    .ok()
                    .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
                    .unwrap_or_else(|| format!("daemon returned HTTP {status}"));
                Err(Error::Api { status, message })
            }
            Err(ureq::Error::Transport(_)) => Err(Error::Unreachable {
                base: self.base.clone(),
            }),
        }
    }
}
