#!/usr/bin/env sh
# Run the whole stack for development: the tasks daemon plus the Tauri app.
# The daemon is started first (backgrounded) and shut down when the app exits.
set -e

cd "$(dirname "$0")/.."

echo "[dev] building tasks-server..."
cargo build -p tasks-server

echo "[dev] starting tasks-server (127.0.0.1:4000)..."
cargo run -p tasks-server &
DAEMON_PID=$!
trap 'echo "[dev] stopping tasks-server..."; kill $DAEMON_PID 2>/dev/null' EXIT INT TERM

pnpm --filter @tasks/tauri tauri dev
