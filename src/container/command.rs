//! Container runtime detection and low-level podman/docker command builders.
//!
//! Finds podman (preferred) or docker on the system PATH and provides
//! a [`Command`] ready to use for container operations.
//!
//! Contains functions that construct a [`tokio::process::Command`] for a specific
//! container operation (images, network, run, stop). These are the building
//! blocks used by higher-level orchestration in [`super::lifecycle`].

use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
    WILDFLY_ADMIN_CONTAINER_REPOSITORY,
};
use crate::error::WadoError;
use crate::label::Label;
use crate::wildfly::Server;
use anyhow::Error;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// ------------------------------------------------------ runtime detection

fn detect_runtime() -> Result<PathBuf, Error> {
    which::which("podman")
        .or_else(|_| which::which("docker"))
        .map_err(|_| WadoError::container_runtime_not_found().into())
}

/// Verifies that a container runtime (podman or docker) is available on the system PATH.
pub fn verify_container_command() -> Result<PathBuf, Error> {
    detect_runtime()
}

/// Creates a new [`Command`] using the detected container runtime.
pub fn container_command() -> anyhow::Result<Command> {
    detect_runtime().map(Command::new)
}

// ------------------------------------------------------ command builders

/// Builds a command to list local wado container images.
pub fn container_images_cmd() -> Command {
    let mut command = container_command().expect("Unable to run docker images/podman images.");
    command
        .arg("images")
        .arg("--filter")
        .arg(format!(
            "reference={}/{}*",
            WILDFLY_ADMIN_CONTAINER_REPOSITORY, WILDFLY_ADMIN_CONTAINER
        ))
        .arg("--format")
        .arg("{{.Repository}}:{{.Tag}}");
    command
}

/// Creates the shared `wado` container network unless it already exists.
///
/// Works with both podman and docker: instead of podman's `network create --ignore`
/// (which docker doesn't support), it checks for the network with `network inspect` first.
pub async fn container_network_cmd() -> anyhow::Result<()> {
    ensure_network(&detect_runtime()?).await
}

async fn ensure_network(runtime: &Path) -> anyhow::Result<()> {
    if network_exists(runtime).await? {
        return Ok(());
    }
    let output = Command::new(runtime)
        .arg("network")
        .arg("create")
        .arg(WILDFLY_ADMIN_CONTAINER)
        .output()
        .await?;
    // Another wado process may have created the network in the meantime.
    if output.status.success() || network_exists(runtime).await? {
        return Ok(());
    }
    Err(WadoError::container_command_failed(
        &format!("Failed to create network {WILDFLY_ADMIN_CONTAINER}"),
        String::from_utf8_lossy(&output.stderr).trim(),
    )
    .into())
}

async fn network_exists(runtime: &Path) -> anyhow::Result<bool> {
    let status = Command::new(runtime)
        .arg("network")
        .arg("inspect")
        .arg(WILDFLY_ADMIN_CONTAINER)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;
    Ok(status.success())
}

/// Builds a `podman run` / `docker run` command with the given container configuration.
///
/// Every container is attached to the shared `wado` network (see [`container_network_cmd`]),
/// so wado containers can reach each other and other containers on that network by name.
///
/// The command is constructed but not executed — callers typically add the image name
/// and any additional arguments before spawning.
pub fn container_run_cmd(
    name: &str,
    ports: Option<&crate::wildfly::Ports>,
    operations: Vec<String>,
    dev: bool,
    topology_name: Option<&str>,
    config: Option<&str>,
) -> Command {
    let mut command = container_command().expect("Unable to run docker run/podman run.");
    command.args(run_args(
        name,
        ports,
        operations,
        dev,
        topology_name,
        config,
    ));
    command
}

/// Builds the arguments for [`container_run_cmd`], independent of the container runtime.
fn run_args(
    name: &str,
    ports: Option<&crate::wildfly::Ports>,
    operations: Vec<String>,
    dev: bool,
    topology_name: Option<&str>,
    config: Option<&str>,
) -> Vec<String> {
    let mut args: Vec<String> = [
        "run",
        "--rm",
        "--detach",
        "--name",
        name,
        "--network",
        WILDFLY_ADMIN_CONTAINER,
    ]
    .map(String::from)
    .to_vec();
    if dev {
        args.push("--pull=always".into());
    }
    if let Some(ports) = ports {
        args.extend([
            "--publish".into(),
            format!("{}:8080", ports.http),
            "--publish".into(),
            format!("{}:9990", ports.management),
        ]);
    }
    if !operations.is_empty() {
        args.extend([
            "--env".into(),
            format!("{}={}", BOOTSTRAP_OPERATIONS_VARIABLE, operations.join(",")),
        ]);
    }
    if let Some(topology) = topology_name {
        args.extend(["--label".into(), Label::Topology.run_arg(topology)]);
    }
    if let Some(config) = config {
        args.extend(["--label".into(), Label::Config.run_arg(config)]);
    }
    args
}

/// Builds a `podman stop` / `docker stop` command for the given container name.
pub fn container_stop_cmd(name: &str) -> Command {
    let mut command = container_command().expect("Unable to run docker stop/podman stop.");
    command.arg("stop").arg(name);
    command
}

// ------------------------------------------------------ related functions

/// Appends `--env SERVERS=...` to the command if servers are provided.
pub fn add_servers(mut command: Command, hostname: &str, servers: Vec<Server>) -> Command {
    if !servers.is_empty() {
        let server_ops = servers
            .iter()
            .map(|server| server.add_server_op(hostname))
            .collect::<Vec<String>>();
        command
            .arg("--env")
            .arg(format!("{}={}", SERVERS_VARIABLE, server_ops.join(",")));
    }
    command
}

/// Creates a podman/docker secret by piping the value to stdin.
pub async fn create_secret(secret_name: &str, secret_value: &str) -> anyhow::Result<()> {
    let mut podman_secret = container_command()?
        .arg("secret")
        .arg("create")
        .arg("--replace")
        .arg(secret_name)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn podman secret");
    if let Some(mut stdin) = podman_secret.stdin.take() {
        stdin.write_all(secret_value.as_bytes()).await?;
    }
    podman_secret.wait().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTALLED_RUNTIMES: [&str; 2] = ["podman", "docker"];

    #[test]
    fn run_args_attach_container_to_wado_network() {
        // Act
        let args = run_args("wado-sa-420", None, vec![], false, None, None);

        // Assert
        let network = args
            .iter()
            .position(|arg| arg == "--network")
            .and_then(|index| args.get(index + 1));
        assert_eq!(network.map(String::as_str), Some(WILDFLY_ADMIN_CONTAINER));
    }

    #[test]
    fn run_args_start_with_detached_named_run() {
        // Act
        let args = run_args("wado-sa-420", None, vec![], false, None, None);

        // Assert
        assert_eq!(
            args[..5],
            ["run", "--rm", "--detach", "--name", "wado-sa-420"]
        );
    }

    #[tokio::test]
    #[ignore = "requires podman and/or docker; run with `cargo test -- --ignored`"]
    async fn ensure_network_is_idempotent_for_each_installed_runtime() {
        // Arrange
        let runtimes: Vec<PathBuf> = INSTALLED_RUNTIMES
            .iter()
            .filter_map(|name| which::which(name).ok())
            .collect();
        assert!(!runtimes.is_empty(), "neither podman nor docker found");

        for runtime in runtimes {
            // Act: the second call must succeed although the network already exists
            ensure_network(&runtime).await.unwrap();
            ensure_network(&runtime).await.unwrap();

            // Assert
            assert!(
                network_exists(&runtime).await.unwrap(),
                "network missing for {}",
                runtime.display()
            );
        }
    }
}
