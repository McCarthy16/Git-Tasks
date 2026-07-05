# HTTP API

The daemon (`tasks-server`) exposes the core as plain data operations over
local HTTP. It listens on **`http://127.0.0.1:4000`** — loopback only, by
design.

The API is a thin shell: each route maps onto exactly one `tasks_core`
operation and returns its result as JSON. No app logic (navigation, selection,
screens) lives in the daemon — that belongs to clients.

## Workspace addressing

The daemon serves any number of workspaces at once and holds no per-workspace
state. Every data route names its workspace with a query parameter:

```
GET /projects?workspace=/absolute/path/to/repo
```

Each request opens a fresh event store over that root. A `workspace` path that
isn't a directory on disk yields `404`.

## Routes

### Meta

| Method & path | Effect |
|---|---|
| `GET /health` | Liveness probe → `{ "status": "ok" }`. |
| `POST /workspaces` | Initialize a `.tasks` tree at `{ "path": ... }` (idempotent). |

### Projects

| Method & path | Effect |
|---|---|
| `GET /projects` | List open projects, oldest first (`&closed=true` lists closed ones instead). |
| `POST /projects` | Create — `{ "name": ... }`. |
| `GET /projects/{id}` | Load one project. |
| `POST /projects/{id}/rename` | `{ "new_name": ... }` |
| `POST /projects/{id}/close` | Close. |
| `POST /projects/{id}/reopen` | Reopen. |

### Tasks

| Method & path | Effect |
|---|---|
| `GET /tasks` | List open tasks, oldest first (`&project_id=` scopes to a project, `&closed=true` lists closed ones instead). |
| `POST /tasks` | Create — `{ "project_id": ..., "name": ... }`. |
| `GET /tasks/{id}` | Load one task. |
| `GET /tasks/{id}/events` | The task's raw event history, oldest first. |
| `POST /tasks/{id}/rename` | `{ "new_name": ... }` |
| `POST /tasks/{id}/move` | `{ "new_project_id": ... }` |
| `POST /tasks/{id}/close` | Close. |
| `POST /tasks/{id}/reopen` | Reopen. |
| `POST /tasks/{id}/status` | `{ "status_id": ... }` (`null` clears). |
| `POST /tasks/{id}/description` | `{ "description": ... }` |

### Statuses

| Method & path | Effect |
|---|---|
| `GET /statuses` | List statuses — seeds included, even on an empty workspace. |
| `POST /statuses` | Create — `{ "name": ..., "kind": ..., "description": ... }`. |
| `POST /statuses/{id}/rename` | `{ "new_name": ... }` |
| `POST /statuses/{id}/description` | `{ "description": ... }` (nullable). |
| `POST /statuses/{id}/kind` | `{ "new_kind": ... }` |
| `POST /statuses/{id}/remove` | Soft-remove. |

Every write returns the freshly rebuilt entity, so clients can render the
result without a follow-up read.

## Errors

Errors return `{ "error": "<message>" }` with a status code mapped from the
core error:

| Condition | Status |
|---|---|
| Unknown workspace root, or `ProjectNotFound` / `TaskNotFound` / `StatusNotFound` | `404` |
| Malformed ID in a request payload | `400` |
| Storage / rebuild failure | `500` |

## Concurrency

Concurrent clients never contend: the daemon is stateless per workspace, and
concurrent writes are safe by the store's design — one immutable, uniquely
named file per event (see [Events & Storage](events-data-store.md)).
