//! Commands — the store-backed operations for each entity.
//!
//! One module per entity ([`project`], [`task`], [`status`]). Each bundles:
//!
//! - the **write** commands — validate intent, append a single event to the
//!   [`EventStore`](crate::events::EventStore), and return the freshly-rebuilt
//!   [`projection`](crate::projections);
//! - the **read** helpers (`load`/`list`/`exists`) the writes use to validate,
//!   and that callers (e.g. the server's view rendering) use to serve reads.
//!
//! Every operation is generic over `impl EventStore`, so the same logic runs
//! against any backend — the filesystem [`FsEventStore`](crate::storage::FsEventStore)
//! that core ships, or any other store a consumer supplies. Reconstruction is
//! seed-blind; only [`status`] deals with seeds, and it does so through the
//! [`storage::seeds`](crate::storage::seeds) overlay rather than knowing about
//! seeds itself.

pub mod project;
pub mod status;
pub mod task;
