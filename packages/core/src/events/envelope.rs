//! The event envelope and the trait every domain's event enum implements.
//!
//! On disk an event looks like:
//!
//! ```json
//! { "id": "<uuidv7>", "type": "created", "payload": { "name": "..." } }
//! ```
//!
//! The `id` is the event's own UUIDv7; `type` and `payload` come from the
//! domain-specific event enum, which is adjacently tagged and flattened into
//! the envelope.

use serde::{Deserialize, Serialize};

use crate::shared::id::EventId;

/// Implemented by every domain's event enum to expose its variant's `type` tag
/// (e.g. `"created"`). This is the same string serde writes as the `type` field
/// in the JSON, and it's appended to the event filename (`<hex>-created.json`)
/// so the store is self-describing on disk.
pub trait EventKind {
    fn event_type(&self) -> &'static str;
}

/// An event wrapping a domain-specific payload `K`.
///
/// `K` is expected to be an adjacently-tagged enum (`#[serde(tag = "type",
/// content = "payload")]`) so that flattening produces the `type`/`payload`
/// fields alongside `id`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event<K> {
    pub id: EventId,
    #[serde(flatten)]
    pub kind: K,
}

impl<K> Event<K> {
    /// Wrap a payload in a freshly-minted, time-ordered event.
    pub fn new(kind: K) -> Self {
        Self {
            id: EventId::new(),
            kind,
        }
    }

    /// Creation time (ms since the Unix epoch) decoded from the event ID.
    pub fn created_at_millis(&self) -> Option<u64> {
        self.id.created_at_millis()
    }
}
