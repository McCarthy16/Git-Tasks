# Events & Data Store Architecture

The app stores all data as an **append-only event log on the filesystem**. There
is no database and no mutable state files. Entity state (a project, a task) is
never stored directly — it is rebuilt by replaying that entity's events in
order.

This design exists primarily to **avoid write conflicts** between concurrent
contributors and across git merges (see [Why this avoids conflicts](#why-this-avoids-conflicts)).

## On-disk layout

Everything lives under a `.tasks/` directory inside the open workspace folder:

```text
.tasks/
  projects/
    project_<hex>/
      events/
        <event-hex>.json
  tasks/
    task_<hex>/
      events/
        <event-hex>.json
```

- `project_<hex>` / `task_<hex>` — the entity ID (UUIDv7, type-prefixed). See [IDs](ids.md).
- `<event-hex>.json` — one file per event, named by the event's own UUIDv7 hex.

The directory is found-or-created on launch (`Store::initialize`), which is
idempotent and safe to run whether or not `.tasks/` already exists.

## The event envelope

Every event, regardless of entity, is wrapped in the same envelope and
serialized as:

```json
{
  "id": "<uuidv7>",
  "type": "created",
  "payload": { "name": "..." }
}
```

- `id` — the event's own UUIDv7. Also encodes the creation timestamp.
- `type` + `payload` — come from the entity-specific event enum, adjacently
  tagged (`#[serde(tag = "type", content = "payload")]`) and flattened into the
  envelope.

## Aggregates

Two entity types exist today. Each is defined by an event enum plus a `replay`
function that folds events into current state.

| Aggregate | Events | State fields |
|-----------|--------|--------------|
| Project   | `created { name }` | `id`, `name`, `created_at_millis` |
| Task      | `created { project_id, name }` | `id`, `project_id`, `name`, `created_at_millis` |

`replay` returns `None` if a history doesn't start with a `created` event, so a
malformed or empty history never produces a half-built entity.

### Relationships live in payloads, not the folder tree

A task's link to its project is stored in the `created` event payload
(`project_id`), **not** in the directory structure. This means a task can later
move between projects via a future `moved` event without moving any files on
disk. The folder tree only ever groups an entity with its own events.

## Reading and writing

- **Write** (`write_event`): create the entity's `events/` dir if needed, write
  the event to `<event-hex>.json`. Each event lands in its own unique file —
  writes never touch an existing file.
- **Read** (`read_events`): list `*.json` in the entity's `events/` dir, sort by
  filename, parse each, replay. A missing directory reads as an empty history.
- **List** (`list_entity_ids`): read the immediate subdirectory names of
  `projects/` or `tasks/`, parse each as an ID, sort. Sorted oldest-first
  because UUIDv7 hex sorts chronologically.

Creating a task first checks that its project exists (`Error::ProjectNotFound`),
so a task can never reference a project with no events on disk.

## Why this avoids conflicts

This is the central design constraint:

1. **One file per event, named by a globally-unique UUIDv7.** Two contributors
   creating events at the same time generate different filenames, so their
   writes never target the same path — even offline, even across machines.
2. **Events are immutable.** Nothing ever edits or deletes an existing event
   file. There is no "current state" file that two writers could clobber.
3. **No shared counter or sequence.** IDs are minted locally with no
   coordination, so there's no central resource to contend on.
4. **Ordering is implied by the ID.** Because UUIDv7 hex sorts chronologically,
   replaying events in filename order reconstructs the true sequence without a
   stored index.

The practical payoff: when two people work in the same workspace (or the same
repo is merged in git), event files from both simply coexist in the `events/`
directory. A git merge of two new event files is a clean add/add with no
conflict, and replay naturally orders them by time.

## Adding a new event type

1. Add a variant to the entity's `*EventKind` enum (e.g. `ProjectEventKind`).
2. Handle that variant in the entity's `replay` to fold it into state.
3. Add a `Store` method that constructs and appends the event via `write_event`.

No migration is required — old event files keep replaying unchanged, and the new
variant only appears in newly-written events.
