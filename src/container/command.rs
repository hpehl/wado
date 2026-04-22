//! Low-level podman/docker command builders.
//!
//! Each function constructs a [`tokio::process::Command`] for a specific
//! container operation (images, network, run, stop). These are the building
//! blocks used by higher-level orchestration in [`super::lifecycle`].

use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, WILDFLY_ADMIN_CONTAINER, WILDFLY_ADMIN_CONTAINER_REPOSITORY,
};
use crate::label::Label;
use std::process::Stdio;
use tokio::process::Command;

use super::runtime::container_command;

pub fn container_images() -> Command {
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

pub async fn container_network() -> anyhow::Result<()> {
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
pub fn container_run(
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

pub fn container_stop(name: &str) -> Command {
    let mut command = container_command().expect("Unable to run docker stop/podman stop.");
    command.arg("stop").arg(name);
    command
}
