use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
    WILDFLY_ADMIN_CONTAINER_REPOSITORY,
};
use crate::label::Label;
use crate::progress::{stderr_reader, summary, Progress};
use crate::wildfly::ServerType::{DomainController, Standalone};
use crate::wildfly::{
    ContainerConfig, ContainerInstance, HasWildFlyContainer, Ports, Server, ServerType,
};
use anyhow::{bail, Error};
use futures::future::join_all;
use indicatif::MultiProgress;
use std::collections::{HashMap, HashSet};
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
    wildfly_containers: Option<&[WildFlyContainer]>,
    name: Option<&str>,
    resolve_ports: bool,
) -> anyhow::Result<Vec<ContainerInstance>> {
    let mut instances = ps_instances(&Label::Id.filter(), |instance| {
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
    })
    .await?;

    if resolve_ports {
        let futures = instances.iter().map(container_ports);
        let results = join_all(futures).await;
        instances = results.into_iter().filter_map(|r| r.ok()).collect();
    }
    Ok(instances)
}

pub async fn containers_by_topology(topology_name: &str) -> anyhow::Result<Vec<ContainerInstance>> {
    ps_instances(&Label::Topology.filter_value(topology_name), |_| true).await
}

pub fn container_run(
    name: &str,
    ports: Option<&Ports>,
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

// ------------------------------------------------------ higher functions

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

pub fn ensure_unique_instances<T>(
    items: &[T],
    copy_fn: fn(&T, Option<u16>, u16) -> T,
    same_type_count_fn: impl Fn(&WildFlyContainer) -> u16,
    all_type_count_fn: impl Fn(&WildFlyContainer) -> u16,
) -> Vec<T>
where
    T: Clone,
    T: HasWildFlyContainer,
{
    let chunks =
        items.chunk_by(|a, b| a.wildfly_container().identifier == b.wildfly_container().identifier);
    let mut result = vec![];
    for chunk in chunks {
        let wc = chunk[0].wildfly_container();
        let same_type = same_type_count_fn(wc);
        let all_type = all_type_count_fn(wc);
        let needs_name_index = chunk.len() > 1 || same_type > 0;
        if needs_name_index || all_type > 0 {
            for (index, item) in chunk.iter().enumerate() {
                let name_index = if needs_name_index {
                    Some(same_type + index as u16)
                } else {
                    None
                };
                let port_offset = all_type + index as u16;
                result.push(copy_fn(item, name_index, port_offset));
            }
        } else {
            result.push(chunk[0].clone());
        }
    }
    result
}

pub async fn get_instance(
    wildfly_containers: Option<&[WildFlyContainer]>,
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
                    .map(|x| x.display_version())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let (Some(name), None) = (name, wildfly_containers) {
            format!("for name '{}'", name)
        } else if let (None, Some(wildfly_containers)) = (name, wildfly_containers) {
            format!(
                "for version '{}'",
                wildfly_containers
                    .iter()
                    .map(|x| x.display_version())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else {
            "".to_string()
        };
        bail!("{} found {}", what, why)
    }
    container_ports(&instances[0]).await
}

pub async fn run_instances<T, F>(instances: &[T], build_command: F) -> anyhow::Result<()>
where
    T: ContainerConfig,
    F: Fn(&T) -> Command,
{
    let names: Vec<&str> = instances.iter().map(|i| i.name()).collect();
    check_name_conflicts(&names).await?;

    let count = instances.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for instance in instances {
        let progress = Progress::new(
            &instance
                .admin_container()
                .wildfly_container
                .display_version(),
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

pub async fn running_counts(
    wildfly_containers: &[WildFlyContainer],
) -> anyhow::Result<HashMap<u16, u16>> {
    let mut seen = HashSet::new();
    let unique: Vec<_> = wildfly_containers
        .iter()
        .filter(|wc| seen.insert(wc.identifier))
        .collect();
    let futures: Vec<_> = unique.iter().map(|wc| running_instance_count(wc)).collect();
    let results = join_all(futures).await;
    let mut counts = HashMap::new();
    for (wc, result) in unique.iter().zip(results) {
        counts.insert(wc.identifier, result?);
    }
    Ok(counts)
}

pub async fn running_counts_by_type(
    server_type: ServerType,
    wildfly_containers: &[WildFlyContainer],
) -> anyhow::Result<HashMap<u16, u16>> {
    let mut seen = HashSet::new();
    let unique: Vec<_> = wildfly_containers
        .iter()
        .filter(|wc| seen.insert(wc.identifier))
        .collect();
    let futures: Vec<_> = unique
        .iter()
        .map(|wc| running_instance_count_by_type(server_type, wc))
        .collect();
    let results = join_all(futures).await;
    let mut counts = HashMap::new();
    for (wc, result) in unique.iter().zip(results) {
        counts.insert(wc.identifier, result?);
    }
    Ok(counts)
}

pub async fn running_instance_count(wildfly_container: &WildFlyContainer) -> anyhow::Result<u16> {
    let instances = container_ps(
        vec![
            ServerType::Standalone,
            ServerType::DomainController,
            ServerType::HostController,
        ],
        Some(std::slice::from_ref(wildfly_container)),
        None,
        false,
    )
    .await?;
    Ok(instances.len() as u16)
}

pub async fn running_instance_count_by_type(
    server_type: ServerType,
    wildfly_container: &WildFlyContainer,
) -> anyhow::Result<u16> {
    let instances = container_ps(
        vec![server_type],
        Some(std::slice::from_ref(wildfly_container)),
        None,
        false,
    )
    .await?;
    Ok(instances.len() as u16)
}

pub async fn stop_containers_by_name(names: &[String]) -> anyhow::Result<()> {
    let count = names.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for name in names {
        let progress = Progress::new(name, name);
        multi_progress.add(progress.bar.clone());
        let mut command = container_stop(name);
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-stop.");
        let name = name.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish(output, Some(&name))
        });
    }

    let status = commands.join_all().await;
    summary("Stopped", "container", count, instant, status);
    Ok(())
}

pub async fn stop_instances(
    server_type: ServerType,
    wildfly_containers: Option<&[WildFlyContainer]>,
    name: Option<&str>,
) -> anyhow::Result<()> {
    let instances = container_ps(vec![server_type], wildfly_containers, name, false).await?;
    let count = instances.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for instance in instances {
        let progress = Progress::new(
            &instance.admin_container.wildfly_container.display_version(),
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

// ------------------------------------------------------ internal helper functions

async fn ps_instances(
    filter: &str,
    predicate: impl Fn(&ContainerInstance) -> bool,
) -> anyhow::Result<Vec<ContainerInstance>> {
    let mut command = container_command()?;
    command
        .arg("ps")
        .arg("--filter")
        .arg(filter)
        .arg("--format")
        .arg(format!(
            "{{{{.ID}}}}|{}|{{{{.Names}}}}|{{{{.Status}}}}|{}|{}",
            Label::Id.format_expr(),
            Label::Topology.format_expr(),
            Label::Config.format_expr(),
        ));
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    let output = String::from_utf8(output.stdout)?;
    let mut instances = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 6
            && let Ok(instance) =
                ContainerInstance::new(parts[1], parts[0], parts[2], parts[3], parts[4], parts[5])
            && predicate(&instance)
        {
            instances.push(instance);
        }
    }
    Ok(instances)
}

async fn check_name_conflicts(names: &[&str]) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("ps").arg("--format").arg("{{.Names}}");
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;
    let running_names: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let conflicts: Vec<&str> = names
        .iter()
        .filter(|n| running_names.contains(**n))
        .copied()
        .collect();
    if !conflicts.is_empty() {
        bail!(
            "Container name(s) already in use: {}. Please retry.",
            conflicts
                .iter()
                .map(|n| format!("'{}'", n))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

// ------------------------------------------------------ verify functions

fn detect_runtime() -> Result<PathBuf, Error> {
    which("podman").or_else(|_| which("docker")).map_err(|_| {
        anyhow::anyhow!("Neither podman nor docker found. Install one of them to continue")
    })
}

pub fn verify_container_command() -> Result<PathBuf, Error> {
    detect_runtime()
}

pub fn container_command() -> anyhow::Result<Command> {
    detect_runtime().map(Command::new)
}
