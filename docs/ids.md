# IDs

Every identifier in the system is a UUID wrapped in a typed newtype. There are
two generation schemes with distinct jobs:

- **UUIDv7** — every normally-created entity and every event. Time-ordered and
  coordination-free.
- **UUIDv5** — seed entities only. Deterministic, derived from a slug, so
  built-in defaults have the same ID on every machine (see [Seeds](seeds.md)).

## Why UUIDv7

UUIDv7 encodes a millisecond Unix timestamp in its most significant bits, which
gives us:

- **Time-ordered**: the hex form sorts lexicographically in chronological
  order. This is load-bearing — event files replay in filename-sort order, and
  entities list oldest-first with a plain sort, no stored index or sequence
  number anywhere.
- **Globally unique**: no coordination needed between machines to avoid
  collisions, which is what makes concurrent offline writes conflict-free.
- **Self-timestamping**: an entity's `created_at_millis` is decoded from its
  first event's ID rather than stored as a field.

Do not use auto-incrementing integers, UUIDv4, or ad-hoc schemes for new
entities.

## Textual format

Identifiers serialize as `<prefix><hex>`, where `<hex>` is the UUID in simple
(non-hyphenated) form:

| ID | Prefix | Example |
|---|---|---|
| Project | `project_` | `project_01932b4a7f3e...` |
| Task | `task_` | `task_01932d8b2a1c...` |
| Status | `status_` | `status_01932f6c9b2d...` |
| Event | *(none)* | `01932d8b2a1c...` |

- **Entity IDs** carry a type prefix, making them self-describing and letting
  parsing reject an ID of the wrong type. The prefixed form is also the
  entity's folder name on disk.
- **Event IDs** are bare hex — they name event files
  (`<hex>-<type>.json`), where the type suffix already says what they are.

## Implementation

All ID types are minted by one macro (`prefixed_id!` in `shared/id.rs`), which
generates the newtype plus its `Display`/`FromStr`/serde implementations —
serialization round-trips through the prefixed string form. Each ID exposes
`created_at_millis()`, which decodes the UUIDv7 timestamp (and returns `None`
for non-v7 UUIDs, i.e. seed IDs).

## Seed IDs (UUIDv5)

Built-in defaults (the shipped statuses) need IDs that are stable across
builds, machines, and users — a task in a shared repo that references the
`Backlog` status must resolve on a teammate's machine too. So seed IDs are
UUID**v5**: a SHA-1 hash of a fixed namespace UUID (`SEED_NAMESPACE`) and the
seed's human-readable slug (`"backlog"`, `"in_progress"`, …). Same slug, same
ID, everywhere.

The trade-off: v5 IDs carry no timestamp, so a seed's `created_at_millis` is
`None` until real (v7) events are written on top of it.
