# IDs

All builds use **UUIDv7** for identifiers.

## Why UUIDv7

UUIDv7 encodes a Unix timestamp in its most significant bits, which gives us:

- **Time-ordered**: IDs sort chronologically by generation time, so they're index- and B-tree-friendly (unlike random UUIDv4).
- **Globally unique**: No coordination needed between services to avoid collisions.
- **No central sequence**: IDs can be generated anywhere — client, server, or worker — without a shared counter.

## Rules

- Use UUIDv7 for all newly generated identifiers across every build.
- Do not use auto-incrementing integers, UUIDv4, or ad-hoc ID schemes for new entities.

## On-the-wire format

Identifiers serialize as `<prefix><hex>`, where `<hex>` is the UUIDv7 in simple
(non-hyphenated) form:

- **Entity IDs** carry a type prefix: `project_<hex>`, `task_<hex>`. The prefix
  makes IDs self-describing and lets parsing reject an ID of the wrong type.
- **Event IDs** are bare UUIDv7 hex with no prefix (they name event files on
  disk — see [Events & Data Store](events-data-store.md)).

Because UUIDv7 encodes a millisecond timestamp in its most significant bits, the
hex form sorts lexicographically in chronological order — which is how event
files are ordered on disk and how entities are listed oldest-first.
