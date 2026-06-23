//! The event-sourcing kernel shared by every domain.
//!
//! This is deliberately thin and domain-agnostic: identifiers ([`id`]), the
//! event envelope ([`event`]), and the filesystem [`store`] helpers. The
//! `projects` and `tasks` domains build their own models, events, and helpers
//! on top of these.

pub mod event;
pub mod id;
pub mod store;
