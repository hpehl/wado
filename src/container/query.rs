//! Query running wado containers.
//!
//! Wraps `podman ps` and `podman inspect` to list, filter, and inspect
//! container instances managed by wado.

use crate::label::Label;
use crate::wildfly::ServerType::DomainController;
use crate::wildfly::{ContainerInstance, Ports, ServerType};
use futures::future::join_all;
use std::collections::BTreeSet;
use std::process::Stdio;
use wildfly_container_versions::WildFlyContainer;

use super::runtime::container_command;

/// Lists running wado containers, filtered by server type, version, and name.
///
/// When `resolve_ports` is true, each container is inspected to determine its
/// actual host port mappings — this adds one `podman inspect` call per container.
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

/// Returns all containers belonging to a specific topology.
pub async fn containers_by_topology(topology_name: &str) -> anyhow::Result<Vec<ContainerInstance>> {
    ps_instances(&Label::Topology.filter_value(topology_name), |_| true).await
}

/// Returns the names of all currently running topologies.
pub async fn running_topology_names() -> anyhow::Result<Vec<String>> {
    let instances = ps_instances(&Label::Topology.filter(), |_| true).await?;
    let names: BTreeSet<String> = instances
        .iter()
        .filter_map(|i| i.topology.clone())
        .collect();
    Ok(names.into_iter().collect())
}

/// Looks up exactly one running container matching the given filters.
///
/// Returns an error if zero or more than one container matches — callers
/// should provide enough filters (version, name) to identify a single instance.
pub async fn get_instance(
    wildfly_containers: Option<&[WildFlyContainer]>,
    name: Option<&str>,
) -> anyhow::Result<ContainerInstance> {
    let instances = container_ps(
        vec![ServerType::Standalone, DomainController],
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
        let why = match (name, wildfly_containers) {
            (Some(name), Some(wcs)) => format!(
                "for name '{}' and version '{}'",
                name,
                wcs.iter()
                    .map(|x| x.display_version())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            (Some(name), None) => format!("for name '{}'", name),
            (None, Some(wcs)) => format!(
                "for version '{}'",
                wcs.iter()
                    .map(|x| x.display_version())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            (None, None) => String::new(),
        };
        anyhow::bail!("{} found {}", what, why)
    }
    container_ports(&instances[0]).await
}

// ------------------------------------------------------ internal

pub(super) async fn container_ports(
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

pub(super) async fn ps_instances(
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
