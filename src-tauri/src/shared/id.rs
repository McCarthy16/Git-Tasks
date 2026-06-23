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

            /// The underlying UUID.
            pub fn as_uuid(&self) -> ::uuid::Uuid {
                self.0
            }

            /// Creation time (ms since the Unix epoch) decoded from the UUIDv7.
            pub fn created_at_millis(&self) -> ::core::option::Option<u64> {
                $crate::shared::id::millis_from_uuid_v7(&self.0)
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
