use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, LABEL_NAME, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
};
use crate::progress::{Progress, summary};
use crate::wildfly::ServerType::{DomainController, Standalone};
use crate::wildfly::{ContainerInstance, HasWildFlyContainer, Ports, Server, ServerType};
use anyhow::{Error, bail};
use futures::future::join_all;
use indicatif::MultiProgress;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::Instant;
use which::which;
use wildfly_container_versions::WildFlyContainer;
// ------------------------------------------------------ container commands (a-z)

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

async fn container_ports(container_instance: &mut ContainerInstance) -> anyhow::Result<()> {
    let mut command = container_command()?;
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

pub async fn container_ps(
    server_types: Vec<ServerType>,
    wildfly_containers: Option<&Vec<WildFlyContainer>>,
    name: Option<&str>,
    resolve_ports: bool,
) -> anyhow::Result<Vec<ContainerInstance>> {
    let mut instances: Vec<ContainerInstance> = vec![];
    let mut command = container_command()?;
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
        let futures = instances.iter_mut().map(container_ports);
        join_all(futures).await;
    }
    Ok(instances)
}

pub fn container_run(name: &str, ports: Option<&Ports>, operations: Vec<String>) -> Command {
    let mut command = container_command().expect("Unable to run docker run/podman run.");
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

pub fn container_stop(name: &str) -> Command {
    let mut command = container_command().expect("Unable to run docker stop/podman stop.");
    command.arg("stop").arg(name);
    command
}

// ------------------------------------------------------ higher functions

pub fn ensure_unique_names<T>(items: &[T], copy_fn: fn(&T, u16) -> T) -> Vec<T>
where
    T: Clone,
    T: HasWildFlyContainer,
{
    let chunks =
        items.chunk_by(|a, b| a.wildfly_container().identifier == b.wildfly_container().identifier);
    let mut unique_names = vec![];
    for chunk in chunks {
        if chunk.len() > 1 {
            for (index, item) in chunk.iter().enumerate() {
                unique_names.push(copy_fn(item, index as u16));
            }
        } else {
            unique_names.push(chunk[0].clone());
        }
    }
    unique_names
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
    let instances = container_ps(
        vec![Standalone, DomainController],
        wildfly_containers,
        name,
        true,
    )
    .await?;
    if instances.is_empty() || instances.len() > 1 {
        let what = if instances.is_empty() {
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
        bail!("{} found {}", what, why)
    }
    let mci = &mut instances[0].clone();
    container_ports(mci).await?;
    Ok(mci.clone())
}

pub async fn stop_instances(
    server_type: ServerType,
    wildfly_containers: Option<&Vec<WildFlyContainer>>,
    name: Option<&str>,
) -> anyhow::Result<()> {
    let instances = container_ps(vec![server_type], wildfly_containers, name, false).await?;
    let count = instances.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for instance in instances {
        let progress = Progress::new(
            &instance.admin_container.wildfly_container.short_version,
            &instance.admin_container.image_name(),
        );
        multi_progress.add(progress.bar.clone());
        let mut command = container_stop(&instance.name);
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-stop.");

        commands.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish(output, Some(&instance.name))
        });
    }

    let status = commands.join_all().await;
    summary("Stopped", "container", count, instant, status);
    Ok(())
}

// ------------------------------------------------------ verify functions

pub fn verify_container_command() -> Result<PathBuf, Error> {
    match which("podman") {
        Ok(p) => Ok(p),
        Err(_) => match which("docker") {
            Ok(p) => Ok(p),
            Err(_) => {
                bail!("podman or docker not found");
            }
        },
    }
}

pub fn container_command() -> anyhow::Result<Command> {
    if let Ok(podman_path) = which("podman") {
        let command = Command::new(podman_path);
        Ok(command)
    } else if let Ok(docker_path) = which("docker") {
        let command = Command::new(docker_path);
        Ok(command)
    } else {
        bail!("podman or docker not found");
    }
}

pub fn container_command_name() -> anyhow::Result<&'static str> {
    if which("podman").is_ok() {
        Ok("podman")
    } else if which("docker").is_ok() {
        Ok("docker")
    } else {
        bail!("podman or docker not found");
    }
}
