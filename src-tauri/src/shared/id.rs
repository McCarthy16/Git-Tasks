//! UUIDv7-based identifiers and the macro domains use to mint their own ID type.
//!
//! Entity IDs carry a type prefix (e.g. `project_`, `task_`) and serialize as
//! `<prefix><hex>`. Event IDs are bare UUIDv7 hex. Because UUIDv7 encodes a
//! millisecond timestamp in its most significant bits, the hex form sorts
//! lexicographically in chronological order — which is how event files are
//! ordered on disk.

use uuid::Uuid;

/// Extract the millisecond Unix timestamp encoded in a UUIDv7.
pub fn millis_from_uuid_v7(uuid: &Uuid) -> Option<u64> {
    let (secs, nanos) = uuid.get_timestamp()?.to_unix();
    Some(secs * 1_000 + (nanos as u64) / 1_000_000)
}

/// Defines a newtype wrapper around a UUIDv7 with a fixed string prefix.
///
/// Fully path-qualified so domains can invoke it after a plain
/// `use crate::shared::id::prefixed_id;`.
macro_rules! prefixed_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(::uuid::Uuid);

        #[allow(dead_code)] // part of the ID API; not every accessor is used yet
        impl $name {
            /// The textual prefix written before the hex (empty for event IDs).
            pub const PREFIX: &'static str = $prefix;

            /// Mint a fresh, time-ordered ID.
            pub fn new() -> Self {
                Self(::uuid::Uuid::now_v7())
            }

            /// Wrap an existing UUID (used for deterministic seed IDs).
            pub fn from_uuid(uuid: ::uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// The underlying UUID.
            pub fn as_uuid(&self) -> ::uuid::Uuid {
                self.0
            }

            /// Creation time (ms since the Unix epoch) decoded from the UUIDv7.
            pub fn created_at_millis(&self) -> ::core::option::Option<u64> {
                $crate::shared::id::millis_from_uuid_v7(&self.0)
            }
        }

        impl ::core::convert::From<::uuid::Uuid> for $name {
            fn from(uuid: ::uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "{}{}", $prefix, self.0.as_simple())
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::error::IdError;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                let hex = s
                    .strip_prefix($prefix)
                    .ok_or($crate::error::IdError::MissingPrefix($prefix))?;
                let uuid = ::uuid::Uuid::parse_str(hex)
                    .map_err(|_| $crate::error::IdError::InvalidUuid)?;
                Ok(Self(uuid))
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                let s = <::std::string::String as ::serde::Deserialize>::deserialize(deserializer)?;
                s.parse().map_err(::serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use prefixed_id;

/// Namespace UUID used to derive deterministic IDs for all seeded entities.
///
/// Combine with a per-domain slug via [`seed_id`] to get a stable,
/// human-meaningful ID that survives across builds and machines.
pub const SEED_NAMESPACE: uuid::Uuid = uuid::Uuid::from_bytes([
    0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x78, 0x90,
    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
]);

/// Derive a deterministic entity ID from a human-readable slug.
///
/// Uses UUID v5 (SHA-1 + [`SEED_NAMESPACE`]) so the same slug always produces
/// the same ID — on every machine, in every build, for every user.
///
/// ```
/// // In a domain module:
/// // let id: StatusId = seed_id("backlog");
/// ```
pub fn seed_id<Id: From<uuid::Uuid>>(slug: &str) -> Id {
    Id::from(uuid::Uuid::new_v5(&SEED_NAMESPACE, slug.as_bytes()))
}

prefixed_id!(
    /// Identifier for a single event file (bare UUIDv7 hex, no prefix).
    EventId,
    ""
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_timestamp() {
        assert!(EventId::new().created_at_millis().is_some());
    }
}
