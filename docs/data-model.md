# Data Model

Three aggregates are modeled today: **Project**, **Task**, and **Status**. Each
is defined by an event enum (the write side), a projection (the read side), and
a `replay` function that folds a stream of events into the projection.

## Project

A container for tasks.

| Event | Payload | Effect |
|---|---|---|
| `created` | `name` | Bootstraps the project. |
| `renamed` | `new_name` | Changes the name. |
| `closed` | — | Marks closed. |
| `reopened` | — | Clears closed. |
| `snapshot` | `name`, `closed` | Sets full state at once (see [Snapshot events](#snapshot-events)). |

Projection: `id`, `name`, `closed`, `created_at_millis`.

## Task

A unit of work. A task always belongs to a project.

| Event | Payload | Effect |
|---|---|---|
| `created` | `project_id`, `name` | Bootstraps the task in a project. |
| `renamed` | `new_name` | Changes the name. |
| `moved` | `new_project_id` | Re-parents the task to another project. |
| `closed` / `reopened` | — | Toggle the closed flag. |
| `description_updated` | `description` | Replaces the (markdown) description. |
| `status_changed` | `status_id` (nullable) | Sets the workflow status; `null` explicitly clears it. |
| `snapshot` | full state | Sets full state at once. |

Projection: `id`, `project_id`, `name`, `description`, `status_id`, `closed`,
`created_at_millis`.

The task→project relationship lives in event payloads (`created.project_id`,
then `moved.new_project_id`), **not** in the folder tree — moving a task never
moves files on disk.

## Status

A workflow state a task can be in. Every status has a semantic **kind** the UI
and logic can rely on regardless of what the status is named:

`unstarted` · `started` · `complete` · `canceled`

| Event | Payload | Effect |
|---|---|---|
| `created` | `name`, `kind`, `description` | Bootstraps a user-created status. |
| `renamed` | `new_name` | Changes the name. |
| `description_updated` | `description` (nullable) | Replaces or clears the description. |
| `kind_changed` | `new_kind` | Changes the semantic kind. |
| `removed` | — | Soft-removes the status (it stays resolvable for history). |
| `snapshot` | full state | Sets full state at once. Seed statuses are expressed as a snapshot. |

Projection: `id`, `name`, `kind`, `description`, `removed`,
`created_at_millis` (`None` for an untouched seed).

Statuses are the one seeded entity: the app ships six defaults (Backlog, Todo,
In Progress, In Review, Complete, Canceled) that exist without anything on
disk — see [Seeds](seeds.md). A seed's history has no `created` event; it
starts with the synthetic seed snapshot and accumulates ordinary events on top.

## Snapshot events

Every entity has a `snapshot` variant that sets all mutable state at once.
During replay a snapshot **bootstraps** the entity if it's the first event, or
**wholesale-overwrites** state if it isn't (preserving the original creation
time). Snapshots are what make seeds expressible as plain events, and they give
the model a compaction/repair escape hatch without breaking the fold.

## Replay rules

- Events fold strictly in ID order (chronological — see [IDs](ids.md)).
- A stream must start with a bootstrapping event (`created` or `snapshot`);
  otherwise `replay` returns `None`, so a malformed history never yields a
  half-built entity.
- `created_at_millis` is decoded from the first bootstrapping event's UUIDv7
  ID, not stored as data.

## Invariants enforced by commands

Writes go through the commands layer, which validates referential integrity
before appending:

- A task can only be created in — or moved to — a project that exists
  (`ProjectNotFound` otherwise).
- A task's status can only be set to a status that exists and isn't removed
  (`StatusNotFound` otherwise); setting `null` clears it.
- Update commands require the target entity to exist (`TaskNotFound`, …).

Each successful command appends exactly one event and returns the freshly
rebuilt projection.
