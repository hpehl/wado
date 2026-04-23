//! Subcommand implementations for the wado CLI.
//!
//! Each submodule implements one CLI subcommand: building images, starting/stopping
//! containers, listing status, opening the console, connecting via CLI, etc.

pub mod build;
pub mod cli;
pub mod completions;
pub mod console;
pub mod dc;
pub mod hc;
pub mod images;
pub(crate) mod lifecycle;
pub mod ps;
pub mod push;
pub mod standalone;
pub mod topology;
