# tasks

> Git-native task and project management that lives inside your repository.

---

## What is tasks?

tasks is local, repo-owned task and project management for developers who want their planning versioned alongside their code. No external services, no accounts, no sync issues. Everything lives in the repo, inside a `.tasks/` directory that commits, branches, and merges with your code.

The data is the product; the interfaces are clients on top of it. A local Rust daemon owns the workspaces, and you work with them through a desktop app ([Tauri](https://tauri.app) + React) or the `tasks` CLI. All data stays on your machine — the daemon binds to loopback only.

---

## What You Get

- **A workspace per repository.** Open any repo and tasks finds (or creates) its `.tasks/` directory. One running daemon serves any number of repos to any number of clients.

- **A desktop app and a CLI.** Manage work visually in the desktop app, or from the terminal with the `tasks` CLI — it discovers the workspace from your current directory (like git does) and speaks `--json` for scripting.

- **Projects and tasks.** Organize work into projects, each holding its tasks. Create, rename, close, and reopen either. Move a task between projects at any time.

- **Workflow statuses.** Tasks move through statuses with semantic kinds — unstarted, started, complete, canceled. Sensible defaults ship built in; rename them, change their kind, or add your own to match how you work.

- **Rich task descriptions.** Every task has a markdown description with a proper rich-text editor, autosaved as you type.

- **A full timeline for every task.** Because every change is recorded as an event, each task shows its complete history — when it was created, renamed, moved, or updated. Past description versions can be reopened and read as rendered snapshots.

- **Task management that travels with the code.** Your board branches when the code branches and merges when it merges. Check out last month's commit and see last month's board. Works offline, works air-gapped.

- **Conflict-free collaboration.** Teammates working in the same workspace never produce a git conflict through normal use — concurrent changes merge cleanly as independent files.

---

## Core Design Principles

### Event Sourced
Nothing is ever mutated. Every action (creating a task, renaming a project, changing a status) is written as an immutable event. State is rebuilt by replaying the history, which gives you a full audit trail and the ability to reconstruct your board at any past commit.

### Conflict Free
Events are append-only and every event is its own uniquely named file. Two contributors working at the same time will never produce a git conflict through normal use — a merge of concurrent work is a clean add/add of new event files.

### Repo Native
Everything commits with your code. Branches, merges, history. Works offline, works air-gapped.

### Local First
The daemon serves workspaces over `127.0.0.1` only. Nothing is exposed to the network, and no cloud is involved anywhere.

---

## Documentation

How it's built lives in [`docs/`](docs/README.md):

| Doc | What it covers |
|---|---|
| [Architecture](docs/architecture.md) | The system shape: workspace layout, the layered core crate, the daemon, and the clients. |
| [Events & Storage](docs/events-data-store.md) | The append-only event store: on-disk layout, the event envelope, and why the design is conflict-free. |
| [Data Model](docs/data-model.md) | The aggregates (Project, Task, Status): their events, projections, replay rules, and invariants. |
| [IDs](docs/ids.md) | Identifier conventions: UUIDv7 everywhere, type prefixes, and deterministic seed IDs. |
| [Seeds](docs/seeds.md) | Built-in defaults served from thin air: seeds as synthetic snapshot events overlaid at read time. |
| [HTTP API](docs/http-api.md) | The daemon's local HTTP interface: workspace addressing, routes, and error mapping. |

---

## Development

Prerequisites: Rust (stable), Node.js with [pnpm](https://pnpm.io), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```sh
pnpm install
pnpm dev          # run the full stack: daemon + desktop app
```

Other scripts:

```sh
pnpm dev:server   # just the daemon (cargo run -p tasks-server)
pnpm dev:app      # just the Tauri app
pnpm build        # build the desktop app
cargo run -p tasks-cli -- --help   # the CLI (binary name: tasks)
cargo test        # run the Rust test suite
```

---

## Status

🚧 Early development
