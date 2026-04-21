use crate::constants::{
    BOOTSTRAP_OPERATIONS_VARIABLE, SERVERS_VARIABLE, WILDFLY_ADMIN_CONTAINER,
    WILDFLY_ADMIN_CONTAINER_REPOSITORY,
};
use crate::label::Label;
use crate::progress::{stderr_reader, summary, Progress};
use crate::wildfly::ServerType::{DomainController, Standalone};
use crate::wildfly::{
    ContainerConfig, ContainerInstance, Ports, ResolvedStart, Server, ServerType, StartSpec,
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

// ------------------------------------------------------ resolve start specs

pub async fn resolve_start_specs(
    server_type: ServerType,
    specs: Vec<StartSpec>,
) -> anyhow::Result<Vec<ResolvedStart>> {
    let has_ports = server_type != ServerType::HostController;

    let needs_query: Vec<&WildFlyContainer> = specs
        .iter()
        .filter(|s| {
            s.custom_name.is_none()
                || (has_ports && (s.custom_http.is_none() || s.custom_management.is_none()))
        })
        .map(|s| &s.admin_container.wildfly_container)
        .collect();

    let mut seen = HashSet::new();
    let unique: Vec<_> = needs_query
        .into_iter()
        .filter(|wc| seen.insert(wc.identifier))
        .collect();
    let futures: Vec<_> = unique
        .iter()
        .map(|wc| running_instance_counts(server_type, wc))
        .collect();
    let results = join_all(futures).await;
    let mut counts: HashMap<u16, (u16, u16)> = HashMap::new();
    for (wc, result) in unique.iter().zip(results) {
        let (same_type, all_types) = result?;
        counts.insert(wc.identifier, (same_type, all_types));
    }

    Ok(resolve_specs_with_counts(has_ports, &specs, &counts))
}

fn resolve_specs_with_counts(
    has_ports: bool,
    specs: &[StartSpec],
    counts: &HashMap<u16, (u16, u16)>,
) -> Vec<ResolvedStart> {
    let mut result = Vec::new();
    let chunks = specs.chunk_by(|a, b| {
        a.admin_container.wildfly_container.identifier
            == b.admin_container.wildfly_container.identifier
    });
    for chunk in chunks {
        let wc = &chunk[0].admin_container.wildfly_container;
        let (same_type, all_types) = counts.get(&wc.identifier).copied().unwrap_or((0, 0));

        let auto_named_count = chunk.iter().filter(|s| s.custom_name.is_none()).count();
        let needs_name_index = auto_named_count > 1 || same_type > 0;

        let mut auto_name_counter: u16 = 0;
        for (position, spec) in chunk.iter().enumerate() {
            let name = match &spec.custom_name {
                Some(custom) => custom.clone(),
                None => {
                    let base = spec.admin_container.container_name();
                    if needs_name_index {
                        let index = same_type + auto_name_counter;
                        auto_name_counter += 1;
                        format!("{}-{}", base, index)
                    } else {
                        base
                    }
                }
            };

            let ports = if has_ports {
                let port_offset = all_types + position as u16;
                let http = spec
                    .custom_http
                    .unwrap_or_else(|| wc.http_port() + port_offset);
                let management = spec
                    .custom_management
                    .unwrap_or_else(|| wc.management_port() + port_offset);
                Some(Ports { http, management })
            } else {
                None
            };

            result.push(ResolvedStart {
                admin_container: spec.admin_container.clone(),
                name,
                ports,
            });
        }
    }
    result
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
        let why = match (name, wildfly_containers) {
            (Some(name), Some(wcs)) => format!(
                "for name '{}' and version '{}'",
                name,
                wcs.iter().map(|x| x.display_version()).collect::<Vec<String>>().join(", ")
            ),
            (Some(name), None) => format!("for name '{}'", name),
            (None, Some(wcs)) => format!(
                "for version '{}'",
                wcs.iter().map(|x| x.display_version()).collect::<Vec<String>>().join(", ")
            ),
            (None, None) => String::new(),
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

pub async fn running_instance_counts(
    server_type: ServerType,
    wildfly_container: &WildFlyContainer,
) -> anyhow::Result<(u16, u16)> {
    let instances = container_ps(
        vec![ServerType::Standalone, ServerType::DomainController],
        Some(std::slice::from_ref(wildfly_container)),
        None,
        false,
    )
    .await?;
    let all_types = instances.len() as u16;
    let same_type = instances
        .iter()
        .filter(|i| i.admin_container.server_type == server_type)
        .count() as u16;
    Ok((same_type, all_types))
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

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::test_helpers::sa_spec;
    use crate::wildfly::{AdminContainer, Ports, StartSpec};

    fn counts(entries: &[(u16, u16, u16)]) -> HashMap<u16, (u16, u16)> {
        entries
            .iter()
            .map(|&(id, same, all)| (id, (same, all)))
            .collect()
    }

    fn resolve(specs: &[StartSpec], count_entries: &[(u16, u16, u16)]) -> Vec<ResolvedStart> {
        resolve_specs_with_counts(true, specs, &counts(count_entries))
    }

    // ------------------------------------------------------ resolve_specs_with_counts

    #[test]
    fn no_running_single_item() {
        let specs = vec![sa_spec("39")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "wado-sa-390");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base));
    }

    #[test]
    fn no_running_multiple_same_version() {
        let specs = vec![sa_spec("39"), sa_spec("39")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "wado-sa-390-0");
        assert_eq!(result[1].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.clone()));
        assert_eq!(result[1].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn same_type_running_single_item() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 1, 1)]);
        assert_eq!(result[0].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn different_type_running_ports_adjusted_name_unchanged() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 0, 1)]);
        assert_eq!(result[0].name, "wado-sa-390");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn different_type_running_multiple_same_version() {
        let specs = vec![sa_spec("39"), sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 0, 1)]);
        assert_eq!(result[0].name, "wado-sa-390-0");
        assert_eq!(result[1].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
        assert_eq!(result[1].ports, Some(base.with_offset(2)));
    }

    #[test]
    fn mixed_running_sa_and_dc() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        // 1 SA running (same_type=1), 2 total (SA + DC, all_type=2)
        let result = resolve(&specs, &[(id, 1, 2)]);
        assert_eq!(result[0].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(2)));
    }

    #[test]
    fn multiple_versions_no_running() {
        let specs = vec![sa_spec("39"), sa_spec("35")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "wado-sa-390");
        assert_eq!(result[1].name, "wado-sa-350");
    }

    #[test]
    fn custom_name_no_adjustment() {
        let mut spec = sa_spec("39");
        spec.custom_name = Some("my-container".to_string());
        let result = resolve(&[spec], &[]);
        assert_eq!(result[0].name, "my-container");
    }

    #[test]
    fn custom_http_only_management_adjusted() {
        let mut spec = sa_spec("39");
        spec.custom_http = Some(9000);
        let id = spec.admin_container.wildfly_container.identifier;
        let result = resolve(&[spec.clone()], &[(id, 0, 1)]);
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(ports.http, 9000);
        assert_eq!(
            ports.management,
            spec.admin_container.wildfly_container.management_port() + 1
        );
    }

    #[test]
    fn custom_management_only_http_adjusted() {
        let mut spec = sa_spec("39");
        spec.custom_management = Some(10000);
        let id = spec.admin_container.wildfly_container.identifier;
        let result = resolve(&[spec.clone()], &[(id, 0, 1)]);
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(
            ports.http,
            spec.admin_container.wildfly_container.http_port() + 1
        );
        assert_eq!(ports.management, 10000);
    }

    #[test]
    fn all_custom_no_adjustment() {
        let mut spec = sa_spec("39");
        spec.custom_name = Some("custom".to_string());
        spec.custom_http = Some(9000);
        spec.custom_management = Some(10000);
        let result = resolve(&[spec], &[(390, 2, 3)]);
        assert_eq!(result[0].name, "custom");
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(ports.http, 9000);
        assert_eq!(ports.management, 10000);
    }

    #[test]
    fn hc_no_ports() {
        let wc = WildFlyContainer::version("39").unwrap();
        let spec = StartSpec {
            admin_container: AdminContainer::new(wc, ServerType::HostController),
            custom_name: None,
            custom_http: None,
            custom_management: None,
        };
        let result = resolve_specs_with_counts(false, &[spec], &HashMap::new());
        assert_eq!(result[0].name, "wado-hc-390");
        assert_eq!(result[0].ports, None);
    }
}
