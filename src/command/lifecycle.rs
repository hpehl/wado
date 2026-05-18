//! Container lifecycle orchestration.
//!
//! Starts and stops containers in parallel with progress tracking.
//! Uses [`tokio::task::JoinSet`] for concurrent operations and
//! [`indicatif::MultiProgress`] for visual feedback.

use crate::args::{start_spec, validate_multiple_versions, versions_argument};
use crate::healthcheck::wait_for_healthy;
use crate::json::CommandResult;
use crate::progress::{CommandStatus, Progress, stderr_reader, summary};
use crate::wildfly::{ContainerConfig, ResolvedStart, ServerType};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::collections::HashSet;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::{WildFlyImage, WildFlyImageRegistry};

use crate::container::{
    container_command, container_ps, container_stop_cmd, resolve_start_specs,
    verify_container_command,
};

/// Verifies the container runtime, extracts versions from CLI args, validates
/// options for multi-version runs, resolves unique names/ports, and converts
/// each [`ResolvedStart`] into the caller's instance type.
pub fn prepare_instances<T>(
    matches: &ArgMatches,
    server_type: ServerType,
    restricted_options: &[&str],
    convert: impl Fn(ResolvedStart) -> T,
    registry: &WildFlyImageRegistry,
) -> anyhow::Result<Vec<T>> {
    verify_container_command()?;
    let wildfly_images = versions_argument(matches);
    if wildfly_images.len() > 1 {
        validate_multiple_versions(matches, restricted_options)?;
    }
    let specs = wildfly_images
        .iter()
        .map(|wc| start_spec(matches, wc, server_type))
        .collect();
    let resolved = block_on(resolve_start_specs(server_type, specs, registry))?;
    Ok(resolved.into_iter().map(convert).collect())
}

/// Starts multiple containers in parallel with progress bars.
///
/// Returns the status of each container operation along with the start time
/// for summary reporting. When `json` is true, progress bars are suppressed.
pub async fn run_instances<T, F>(
    instances: &[T],
    build_command: F,
    json: bool,
) -> anyhow::Result<(Vec<CommandStatus>, Instant)>
where
    T: ContainerConfig,
    F: Fn(&T) -> Command,
{
    let names: Vec<&str> = instances.iter().map(|i| i.name()).collect();
    check_name_conflicts(&names).await?;

    let instant = Instant::now();
    let multi_progress = if json {
        None
    } else {
        Some(MultiProgress::new())
    };
    let mut commands = JoinSet::new();

    for instance in instances {
        let progress = create_progress(
            &multi_progress,
            &instance.admin_image().wildfly_image.short_name(),
            &instance.admin_image().image_name(),
        );
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
        if !json {
            tokio::spawn(async move {
                progress_clone.trace_progress(stderr).await;
            });
        }
    }

    let status = commands.join_all().await;
    Ok((status, instant))
}

/// Polls management interfaces in parallel for all successfully started containers.
///
/// Updates each [`CommandStatus`] based on whether the health check succeeded
/// or timed out. Containers that failed to start are skipped.
pub async fn wait_for_instances(status: &mut [CommandStatus], json: bool) {
    let multi_progress = if json {
        None
    } else {
        Some(MultiProgress::new())
    };
    let mut health_checks = JoinSet::new();

    for s in status.iter() {
        if !s.success {
            continue;
        }
        let mgmt_port = match s.management {
            Some(port) => port,
            None => continue,
        };
        let progress = create_progress(&multi_progress, &s.identifier, &s.identifier);
        let identifier = s.identifier.clone();
        health_checks.spawn(async move {
            let healthy = wait_for_healthy(mgmt_port, &progress).await;
            if healthy {
                progress.finish_healthy();
            } else {
                progress.finish_unhealthy();
            }
            (identifier, healthy)
        });
    }

    let results = health_checks.join_all().await;
    for (identifier, healthy) in results {
        if let Some(s) = status.iter_mut().find(|s| s.identifier == identifier)
            && !healthy
        {
            *s = s.clone().with_health_failure();
        }
    }
}

/// Applies port information to command statuses by matching container names.
pub fn apply_ports(
    status: Vec<CommandStatus>,
    port_map: &[(String, u16, u16)],
) -> Vec<CommandStatus> {
    status
        .into_iter()
        .map(|s| {
            if let Some((_, http, mgmt)) = port_map.iter().find(|(n, _, _)| *n == s.identifier) {
                s.with_ports(*http, *mgmt)
            } else {
                s
            }
        })
        .collect()
}

/// Stops containers matching the server type, version, and name from CLI args.
pub fn stop_containers_by_server_type(
    server_type: ServerType,
    matches: &ArgMatches,
    registry: &WildFlyImageRegistry,
    json: bool,
) -> anyhow::Result<Vec<CommandStatus>> {
    verify_container_command()?;
    let wildfly_images = matches.get_one::<Vec<WildFlyImage>>("wildfly-version");
    let name = matches.get_one::<String>("name").map(|s| s.as_str());
    block_on(async {
        let instances = container_ps(
            vec![server_type],
            wildfly_images.map(|v| v.as_slice()),
            name,
            false,
            registry,
        )
        .await?;
        let count = instances.len();
        let instant = Instant::now();
        let multi_progress = if json {
            None
        } else {
            Some(MultiProgress::new())
        };
        let mut commands = JoinSet::new();

        for instance in instances {
            let progress = create_progress(
                &multi_progress,
                &instance.admin_image.wildfly_image.short_name(),
                &instance.admin_image.image_name(),
            );
            let mut command = container_stop_cmd(&instance.name);
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
        if !json {
            summary("Stopped", "container", count, instant, status.clone());
        }
        Ok(status)
    })
}

/// Stops multiple containers by their names.
pub async fn stop_containers_by_name(
    names: &[String],
    json: bool,
) -> anyhow::Result<Vec<CommandStatus>> {
    let count = names.len();
    let instant = Instant::now();
    let multi_progress = if json {
        None
    } else {
        Some(MultiProgress::new())
    };
    let mut commands = JoinSet::new();

    for name in names {
        let progress = create_progress(&multi_progress, name, name);
        let mut command = container_stop_cmd(name);
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
    if !json {
        summary("Stopped", "container", count, instant, status.clone());
    }
    Ok(status)
}

// ------------------------------------------------------ json helpers

fn status_to_json(status: &[CommandStatus]) -> Vec<CommandResult> {
    status
        .iter()
        .map(|s| {
            if s.success {
                CommandResult::success(&s.identifier, s.http, s.management)
            } else {
                CommandResult::error(&s.identifier, &s.error_message)
            }
        })
        .collect()
}

pub fn print_json_results(status: &[CommandStatus]) {
    let results = status_to_json(status);
    println!("{}", serde_json::to_string(&results).unwrap_or_default());
}

// ------------------------------------------------------ internal

/// Creates a progress bar, joining a [`MultiProgress`] group if present,
/// or returning a hidden no-op progress bar for JSON mode.
fn create_progress(
    multi_progress: &Option<MultiProgress>,
    prefix: &str,
    image_name: &str,
) -> Progress {
    match multi_progress {
        Some(mp) => Progress::join(mp, prefix, image_name),
        None => Progress::hidden(prefix, image_name),
    }
}

/// Checks for name collisions against all running containers, not just wado-managed ones.
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
