//! Container runtime detection.
//!
//! Finds podman (preferred) or docker on the system PATH and provides
//! a [`Command`] ready to use for container operations.

use anyhow::Error;
use std::path::PathBuf;
use tokio::process::Command;
use which::which;

fn detect_runtime() -> Result<PathBuf, Error> {
    which("podman").or_else(|_| which("docker")).map_err(|_| {
        anyhow::anyhow!("Neither podman nor docker found. Install one of them to continue")
    })
}

/// Verifies that a container runtime (podman or docker) is available on the system PATH.
pub fn verify_container_command() -> Result<PathBuf, Error> {
    detect_runtime()
}

/// Creates a new [`Command`] using the detected container runtime.
pub fn container_command() -> anyhow::Result<Command> {
    detect_runtime().map(Command::new)
}
