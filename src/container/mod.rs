//! Container management for wado.
//!
//! This module handles all interactions with the container runtime (podman/docker):
//! building commands, querying running containers, resolving names and ports,
//! and orchestrating container lifecycle operations.

mod command;
mod completion;
mod lifecycle;
mod query;
mod resolve;

pub use command::*;
pub use completion::*;
pub use lifecycle::*;
pub use query::*;
pub use resolve::*;
