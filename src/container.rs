use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, LABEL_NAME, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
    WILDFLY_ADMIN_CONTAINER_REPOSITORY,
};
use crate::progress::{Progress, stderr_reader, summary};
use crate::wildfly::ServerType::{DomainController, Standalone};
use crate::wildfly::{
    ContainerConfig, ContainerInstance, HasWildFlyContainer, Ports, Server, ServerType,
};
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

async fn container_ports(
    container_instance: &ContainerInstance,
) -> anyhow::Result<ContainerInstance> {
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
    let ports = if output.status.success() {
        let output = String::from_utf8(output.stdout)?;
        let parts = output.trim().split("|").collect::<Vec<&str>>();
        if parts.len() == 2 {
            let http = parts[0].parse::<u16>()?;
            let management = parts[1].parse::<u16>()?;
            Some(Ports { http, management })
        } else {
            container_instance.ports.clone()
        }
    } else {
        None
    };
    Ok(ContainerInstance {
        ports,
        ..container_instance.clone()
    })
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
        let futures = instances.iter().map(container_ports);
        let results = join_all(futures).await;
        instances = results.into_iter().filter_map(|r| r.ok()).collect();
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

pub async fn run_instances<T, F>(instances: &[T], build_command: F) -> anyhow::Result<()>
where
    T: ContainerConfig,
    F: Fn(&T) -> Command,
{
    let count = instances.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for instance in instances {
        let progress = Progress::new(
            &instance.admin_container().wildfly_container.short_version,
            &instance.admin_container().image_name(),
        );
        multi_progress.add(progress.bar.clone());
        let mut child = build_command(instance)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-run.");

        let stderr = stderr_reader(&mut child);
        let name = instance.name().to_string();
        let progress_clone = progress.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish(output, Some(&name))
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stderr).await;
        });
    }

    let status = commands.join_all().await;
    summary("Started", "container", count, instant, status);
    Ok(())
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
    container_ports(&instances[0]).await
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

fn detect_runtime() -> Result<PathBuf, Error> {
    which("podman")
        .or_else(|_| which("docker"))
        .map_err(|_| anyhow::anyhow!("podman or docker not found"))
}

pub fn verify_container_command() -> Result<PathBuf, Error> {
    detect_runtime()
}

pub fn container_command() -> anyhow::Result<Command> {
    detect_runtime().map(Command::new)
}
