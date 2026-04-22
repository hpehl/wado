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
use crate::label::Label;
use crate::wildfly::Server;
use anyhow::Error;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

fn detect_runtime() -> Result<PathBuf, Error> {
    which::which("podman")
        .or_else(|_| which::which("docker"))
        .map_err(|_| {
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

pub async fn container_network_cmd() -> anyhow::Result<()> {
    let mut network_command = container_command()?;
    network_command
        .arg("network")
        .arg("create")
        .arg("--ignore")
        .arg(WILDFLY_ADMIN_CONTAINER);
    let network_child = network_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Unable to run podman-network.");
    network_child.wait_with_output().await?;
    Ok(())
}

/// Builds a `podman run` / `docker run` command with the given container configuration.
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
    command
        .arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--name")
        .arg(name);
    if dev {
        command.arg("--pull=always");
    }
    if let Some(ports) = ports {
        command
            .arg("--publish")
            .arg(format!("{}:8080", ports.http))
            .arg("--publish")
            .arg(format!("{}:9990", ports.management));
    }
    if !operations.is_empty() {
        command.arg("--env").arg(format!(
            "{}={}",
            BOOTSTRAP_OPERATIONS_VARIABLE,
            operations.join(",")
        ));
    }
    if let Some(topology) = topology_name {
        command
            .arg("--label")
            .arg(Label::Topology.run_arg(topology));
    }
    if let Some(config) = config {
        command.arg("--label").arg(Label::Config.run_arg(config));
    }
    command
}

pub fn container_stop_cmd(name: &str) -> Command {
    let mut command = container_command().expect("Unable to run docker stop/podman stop.");
    command.arg("stop").arg(name);
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
