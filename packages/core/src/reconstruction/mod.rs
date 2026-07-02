//! Reconstruction — building projections from events.
//!
//! The process of taking an event stream and folding it, in chronological
//! order, into a [`projection`](crate::projections). Each entity exposes a
//! `replay` function that consumes its events and produces the fully
//! reconstructed object:
//!
//! - [`project::replay`] — events → [`Project`](crate::projections::Project).
//! - [`task::replay`] — events → [`Task`](crate::projections::Task).
//! - [`status::replay`] — events → [`Status`](crate::projections::Status).
//!
//! Every entity's events include a `Snapshot` variant that sets its full state
//! at once; the fold applies it as a wholesale overwrite (or a bootstrap, if
//! it's the first event), preserving the original creation time.
//!
//! Reconstruction is a *blind fold*: it never knows whether a stream includes a
//! seed. Built-in seeds are assembled into the stream by the
//! [`storage`](crate::storage) layer (see [`storage::seeds`](crate::storage::seeds));
//! by the time events reach `replay`, a seed is just an ordinary leading
//! snapshot event. [`status`] declares its seeds via
//! [`Seeded`](crate::storage::seeds::Seeded).

pub mod project;
pub mod status;
pub mod task;
