# Seeds

A **seed** is a built-in default that exists even though nothing was ever
written for it. The app ships with six workflow statuses (Backlog, Todo, In
Progress, In Review, Complete, Canceled) that appear in every workspace — with
zero files on disk.

## The idea: a seed is just an event

Rather than special-casing defaults throughout the app, a seed is expressed as
a synthetic **snapshot event** that the storage layer *assembles into the
entity's stream at read time*. Everything above storage — reconstruction,
commands, the server, the UI — folds that stream without ever learning a seed
was involved. Seeds are a storage concept, owned entirely by
`storage::seeds`; no other layer knows they exist.

```
on disk:            (nothing)
logical stream:     [ snapshot("Backlog", unstarted) ]        ← synthetic
                                                                 (assembled at read time)

after a rename:     .tasks/statuses/status_<hex>/events/<hex>-renamed.json
logical stream:     [ snapshot("Backlog", unstarted), renamed("Ice Box") ]
```

The seed snapshot itself is **never persisted** — it is re-overlaid on every
read. Only the user's changes on top of it are real files.

## Deterministic identity

Seed IDs must be identical on every machine: a task in a shared repo that
references the Backlog status has to resolve on a teammate's checkout. So a
seed's ID is a UUID**v5**, derived from a fixed namespace and the seed's slug
(`"backlog"`, `"in_progress"`, …) — same slug, same ID, in every build for
every user. See [IDs](ids.md#seed-ids-uuidv5).

Because v5 IDs carry no timestamp, an untouched seed's `created_at_millis` is
`None`; it stays `None` until a real (v7) event is written on top.

## Behavior that falls out

- **Listing includes seeds.** `stream_ids` returns all declared seeds first (in
  canonical slug order), then on-disk entities oldest-first — so a fresh
  workspace already has a full status set. A seed that has accumulated disk
  events is not double-listed.
- **Customizing a seed writes ordinary events.** Renaming Backlog appends a
  normal `renamed` event to the seed's stream; the seed snapshot still leads
  the stream at read time and the rename folds on top.
- **Seeds have no `created` event.** A seed status's history starts with the
  synthetic snapshot; only user-created statuses have a real `created` event.
- **Commands accept seed IDs transparently.** Setting a task's status to a seed
  works with an empty `.tasks/statuses/` — existence checks read through the
  seed overlay.

## The machinery

Two halves in `storage::seeds`:

- **The engine** — generic, entity-agnostic. An entity opts in by implementing
  the `Seeded` trait: its ID and event-kind types, its collection, its slugs
  (`seed_slugs`), and each slug's full-state snapshot payload
  (`seed_snapshot`). Free functions derive the deterministic ID
  (`seed_id_for`) and the synthetic event (`seed_event`).
- **The overlay** — seed-aware reads. `load_stream` prepends the synthetic
  snapshot when (and only when) the requested ID matches a declared seed, then
  appends whatever is on disk; `stream_ids` merges declared seeds with on-disk
  entities.

Today only `Status` implements `Seeded` (in `reconstruction/status.rs`), but
the engine is ready for any entity that needs shipped defaults.
