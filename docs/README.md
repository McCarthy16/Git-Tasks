# Documentation

Implementation documentation for tasks. The [root README](../README.md) covers
what the product is and why; these docs cover how it's built.

## Contents

| Doc | What it covers |
|---|---|
| [Architecture](architecture.md) | The system shape: workspace layout, the layered core crate, the daemon, and the clients. |
| [Events & Storage](events-data-store.md) | The append-only event store: on-disk layout, the event envelope, the `EventStore` trait, and why the design is conflict-free. |
| [Data Model](data-model.md) | The aggregates (Project, Task, Status): their events, projections, replay rules, and command-enforced invariants. |
| [IDs](ids.md) | Identifier conventions: UUIDv7 everywhere, type prefixes, and deterministic UUIDv5 seed IDs. |
| [Seeds](seeds.md) | Built-in defaults served from thin air: seeds as synthetic snapshot events overlaid at read time. |
| [HTTP API](http-api.md) | The daemon's local HTTP interface: workspace addressing, routes, and error mapping. |

## Conventions

Add new documentation as Markdown files in this directory, and link each one
from the table above and from the doc map in the root README.
