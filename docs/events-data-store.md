# Events & Storage

All data is an **append-only event log on the filesystem**. There is no
database and no mutable state files. Entity state (a project, a task, a status)
is never stored directly — it is rebuilt by replaying that entity's events in
order.

This design exists primarily to **avoid write conflicts** between concurrent
contributors and across git merges (see
[Why this avoids conflicts](#why-this-avoids-conflicts)).

## On-disk layout

Everything lives under a `.tasks/` directory inside the workspace (repo)
folder:

```text
.tasks/
  projects/
    project_<hex>/
      events/
        <event-hex>-<type>.json
  tasks/
    task_<hex>/
      events/
        <event-hex>-<type>.json
  statuses/
    status_<hex>/
      events/
        <event-hex>-<type>.json
```

- The top-level folders are **collections** — one per entity type.
- `project_<hex>` / `task_<hex>` / `status_<hex>` — the entity's typed ID
  (UUIDv7 hex with a type prefix, see [IDs](ids.md)). Each entity owns an
  append-only **stream** of events.
- `<event-hex>-<type>.json` — one file per event. The UUIDv7 hex prefix keeps
  writes unique and, because UUIDv7 sorts chronologically, sorting filenames
  replays events in the order they happened. The `type` suffix (e.g.
  `-created.json`, `-renamed.json`) makes the store self-describing on disk.

Entities sit as flat sibling folders. Relationships (like a task's project)
live in event payloads, not the folder structure, so a task can move between
projects without touching the file tree.

The `.tasks` tree is found-or-created on workspace init (`FsEventStore::ensure`),
which is idempotent and safe to run whether or not `.tasks/` already exists.

## The event envelope

Every event, regardless of entity, is wrapped in the same envelope and
serialized as:

```json
{
  "id": "<uuidv7-hex>",
  "type": "created",
  "payload": { "name": "..." }
}
```

- `id` — the event's own UUIDv7, which also encodes the creation timestamp.
- `type` + `payload` — come from the entity-specific event enum, adjacently
  tagged (`#[serde(tag = "type", content = "payload")]`) and flattened into the
  envelope.

The per-entity event enums and what each variant does are documented in
[Data Model](data-model.md).

## The `EventStore` trait

Storage is defined by a trait (`events::store::EventStore`), not a backend.
It is deliberately domain-agnostic — generic over the event payload and the
entity ID type — so a single implementation serves every entity:

- `append(collection, id, event)` — add one event to an entity's stream.
- `read(collection, id)` — the full stream, chronological. An entity with no
  events reads as empty ("empty" means "does not exist"). A missing directory
  is not an error.
- `list_ids(collection)` — every entity with a stream, oldest first (UUIDv7 hex
  sorts chronologically, so a plain sort is a time sort).

Core ships two implementations: `FsEventStore` (the real, filesystem-backed
store described above) and `InMemoryEventStore` (a disk-free double, compiled
only for tests / the `test-util` feature). Commands are generic over
`impl EventStore`, so all domain logic runs unchanged against either.

## Writing

Writes only ever create new files. A write creates the entity's `events/`
directory if needed and writes the event to its own uniquely-named file —
nothing edits or deletes an existing event, and there is no "current state"
file to contend on.

**One deliberate exception:** because the filename is derived from the event's
ID and type, re-appending an event with the *same* event ID overwrites that one
file instead of growing the stream. The commands layer uses this for
session-scoped deduplication (`rename_in_place`, `update_description_in_place`):
the frontend passes back the event ID it minted for this editing session, so
repeated saves of a task description collapse into one event instead of one per
keystroke-debounce. This never mutates history written by anyone else — a
client only ever overwrites the event it itself just created.

## Reading

Reads list the entity's `events/` directory, sort by filename (= chronological
order), parse each file, and hand the stream to `replay` to fold into a
projection. For seeded entities (statuses), the storage layer first overlays
any built-in seed snapshot at the head of the stream — see [Seeds](seeds.md).

`replay` returns `None` unless the history starts with a bootstrapping event
(`created`, or a `snapshot`), so a malformed or empty history never produces a
half-built entity.

## Why this avoids conflicts

This is the central design constraint:

1. **One file per event, named by a globally-unique UUIDv7.** Two contributors
   creating events at the same time generate different filenames, so their
   writes never target the same path — even offline, even across machines.
2. **Events are immutable.** Nothing edits or deletes an existing event file,
   and there is no state file two writers could clobber.
3. **No shared counter or sequence.** IDs are minted locally with no
   coordination, so there is no central resource to contend on.
4. **Ordering is implied by the ID.** UUIDv7 hex sorts chronologically, so
   replaying in filename order reconstructs the true sequence without a stored
   index.

The practical payoff: when two people work in the same workspace (or the same
repo is merged in git), event files from both simply coexist in the `events/`
directory. A git merge of concurrent work is a clean add/add of new files with
no conflict, and replay naturally orders them by time.

## Adding a new event type

1. Add a variant to the entity's `*EventKind` enum (e.g. `TaskEventKind`) and
   map it to a `type` string in its `EventKind::event_type` impl.
2. Handle the variant in the entity's `replay` to fold it into state.
3. Add a command in `commands::<entity>` that validates intent, constructs the
   event, and appends it via the store.

No migration is required — old event files keep replaying unchanged, and the
new variant only appears in newly-written events.
