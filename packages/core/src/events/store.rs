//! The `EventStore` interface — the contract the storage layer implements.
//!
//! This trait is the seam between the event log and everything that reads or
//! writes it. It speaks only in [`Event`]s: callers append events and read them
//! back, without knowing whether they live on disk, in memory, or anywhere
//! else. The [`storage`](crate::storage) layer provides the concrete,
//! filesystem-backed implementation; tests (and future backends) are free to
//! provide their own.
//!
//! Events are grouped into *collections* — one per entity type, named by the
//! domain (e.g. `"projects"`, `"tasks"`, `"statuses"`). Within a collection each
//! entity owns an append-only *stream* of events keyed by its ID. The store is
//! deliberately domain-agnostic: it is generic over the event payload `K` and
//! the entity ID type, so a single implementation serves every domain.

use std::fmt::Display;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::Result;
use crate::events::envelope::{Event, EventKind};

/// An append-only store of domain events, abstracted from how they are stored.
///
/// The rest of the application depends on this trait rather than on any
/// particular backend, so the backend can be swapped without touching the
/// layers above. Core ships one implementation — the filesystem-backed
/// [`FsEventStore`](crate::storage::FsEventStore); a consumer (e.g. the
/// daemon/server) is free to supply another, such as an in-memory double for
/// its own tests.
pub trait EventStore {
    /// Append one event to the stream for `id` within `collection`.
    ///
    /// Events are immutable and uniquely named, so appends never overwrite an
    /// existing event; the stream only ever grows.
    fn append<K>(&self, collection: &str, id: impl Display, event: &Event<K>) -> Result<()>
    where
        K: Serialize + EventKind;

    /// Read the full stream for `id` within `collection`, in chronological
    /// order. An entity with no stored events yields an empty vec (callers treat
    /// "empty" as "does not exist").
    fn read<K>(&self, collection: &str, id: impl Display) -> Result<Vec<Event<K>>>
    where
        K: DeserializeOwned;

    /// List the IDs of every entity that has a stream in `collection`, oldest
    /// first. Names that don't parse as `Id` are skipped; an absent collection
    /// yields an empty vec.
    fn list_ids<Id>(&self, collection: &str) -> Result<Vec<Id>>
    where
        Id: FromStr + Ord;
}
