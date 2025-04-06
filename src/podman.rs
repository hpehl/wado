use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, LABEL_NAME, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
};
use crate::wildfly::ServerType::{DomainController, Standalone};
use crate::wildfly::{ContainerInstance, Ports, Server, ServerType};
use anyhow::{Context, Error, bail};
use futures::future::join_all;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use which::which;
use wildfly_container_versions::WildFlyContainer;

pub async fn create_network() -> anyhow::Result<()> {
    let mut network_command = Command::new("podman");
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

pub fn podman_run(name: &str, ports: Option<&Ports>, operations: Vec<String>) -> Command {
    let mut command = Command::new("podman");
    command
        .arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--name")
        .arg(name);
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
    command
}

pub fn podman_stop(name: &str) -> Command {
    let mut command = Command::new("podman");
    command.arg("stop").arg(name);
    command
}

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

pub async fn get_instance(
    wildfly_containers: Option<&Vec<WildFlyContainer>>,
    name: Option<&str>,
) -> anyhow::Result<ContainerInstance> {
    let instances = podman_ps(
        vec![Standalone, DomainController],
        wildfly_containers,
        name,
        true,
    )
    .await?;
    if instances.len() == 0 || instances.len() > 1 {
        let what = if instances.len() == 0 {
            "No container"
        } else {
            "Multiple containers"
        };
        let why = if let (Some(name), Some(wildfly_containers)) = (name, wildfly_containers) {
            format!(
                "for name '{}' and version '{}'",
                name,
                wildfly_containers
                    .iter()
                    .map(|x| x.short_version.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )
        } else if let (Some(name), None) = (name, wildfly_containers) {
            format!("for name '{}'", name)
        } else if let (None, Some(wildfly_containers)) = (name, wildfly_containers) {
            format!(
                "for version '{}'",
                wildfly_containers
                    .iter()
                    .map(|x| x.short_version.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )
        } else {
            "".to_string()
        };
        if instances.len() == 0 {}
        bail!("{} found {}", what, why)
    }
    let mci = &mut instances[0].clone();
    podman_ports(mci).await?;
    Ok(mci.clone())
}

pub async fn podman_ps(
    server_types: Vec<ServerType>,
    wildfly_containers: Option<&Vec<WildFlyContainer>>,
    name: Option<&str>,
    resolve_ports: bool,
) -> anyhow::Result<Vec<ContainerInstance>> {
    let mut instances: Vec<ContainerInstance> = vec![];
    let mut command = Command::new("podman");
    command
        .arg("ps")
        .arg("--filter")
        .arg(format!("label={}", LABEL_NAME))
        .arg("--format")
        .arg(format!(
            "{{{{.ID}}}}|{{{{index .Labels \"{}\"}}}}|{{{{.Names}}}}|{{{{.Status}}}}",
            LABEL_NAME
        ));
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    let output = String::from_utf8(output.stdout)?;
    for line in output.lines() {
        let parts: Vec<&str> = line.split("|").collect();
        if parts.len() == 4 {
            let container_id = parts[0];
            let identifier = parts[1];
            let name = parts[2];
            let status = parts[3];
            if let Ok(instance) = ContainerInstance::new(identifier, container_id, name, status) {
                instances.push(instance);
            }
        }
    }

    instances.retain(|instance| {
        let server_type_match = server_types.contains(&instance.admin_container.server_type);
        let version_match = if let Some(versions) = &wildfly_containers {
            versions.contains(&instance.admin_container.wildfly_container)
        } else {
            true
        };
        let name_match = if let Some(name) = name {
            name == instance.name
        } else {
            true
        };
        server_type_match && version_match && name_match
    });

    if resolve_ports {
        let futures = instances.iter_mut().map(move |x| podman_ports(x));
        join_all(futures).await;
    }
    Ok(instances)
}

async fn podman_ports(container_instance: &mut ContainerInstance) -> anyhow::Result<()> {
    let mut command = Command::new("podman");
    command.arg("inspect")
            .arg("--format")
            .arg("{{ (index (index .NetworkSettings.Ports \"8080/tcp\") 0).HostPort }}|{{ (index (index .NetworkSettings.Ports \"9990/tcp\") 0).HostPort }}")
            .arg(container_instance.container_id.as_str());
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    if output.status.success() {
        let output = String::from_utf8(output.stdout)?;
        let ports = output.trim().split("|").collect::<Vec<&str>>();
        if ports.len() == 2 {
            let http = ports[0].parse::<u16>()?;
            let management = ports[1].parse::<u16>()?;
            container_instance.ports = Some(Ports { http, management });
        }
    } else {
        container_instance.ports = None;
    }
    Ok(())
}

pub fn verify_podman() -> Result<PathBuf, Error> {
    which("podman").with_context(|| "podman not found")
}
