//! The application layer: the server-driven UI state machine.
//!
//! The frontend is a thin renderer. It holds no routing or domain state of its
//! own — it asks for a [`view::View`] to draw and sends [`action::Action`]s
//! back. [`state::AppState`] reduces those actions against the open workspace
//! and selected project, then re-renders.

pub mod action;
pub mod state;
pub mod view;
