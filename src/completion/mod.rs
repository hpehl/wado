//! Shell completion providers for CLI arguments.
//!
//! This module groups all tab-completion logic used by the CLI:
//! version completions, running container completions, and topology completions.

mod container;
mod version;

pub use container::*;
pub use version::*;
