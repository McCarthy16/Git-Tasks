//! Shared foundation primitives used across every layer.
//!
//! These aren't part of the event-sourcing flow themselves — they're the
//! building blocks (typed IDs, seed derivation) the other layers are defined
//! in terms of.

pub mod id;
