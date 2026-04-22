//! Container lifecycle orchestration.
//!
//! Starts and stops containers in parallel with progress tracking.
//! Uses [`tokio::task::JoinSet`] for concurrent operations and
//! [`indicatif::MultiProgress`] for visual feedback.

use crate::args::{start_spec, validate_multiple_versions, versions_argument};
use crate::progress::{Progress, stderr_reader, summary};
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
use wildfly_container_versions::WildFlyContainer;

use super::command::{container_command, container_stop_cmd, verify_container_command};
use super::query::container_ps;
use super::resolve::resolve_start_specs;

/// Verifies the container runtime, extracts versions from CLI args, validates
/// options for multi-version runs, resolves unique names/ports, and converts
/// each [`ResolvedStart`] into the caller's instance type.
pub fn prepare_instances<T>(
    matches: &ArgMatches,
    server_type: ServerType,
    restricted_options: &[&str],
    convert: impl Fn(ResolvedStart) -> T,
) -> anyhow::Result<Vec<T>> {
    verify_container_command()?;
    let wildfly_containers = versions_argument(matches);
    if wildfly_containers.len() > 1 {
        validate_multiple_versions(matches, restricted_options)?;
    }
    let specs = wildfly_containers
        .iter()
        .map(|wc| start_spec(matches, wc, server_type))
        .collect();
    let resolved = block_on(resolve_start_specs(server_type, specs))?;
    Ok(resolved.into_iter().map(convert).collect())
}

/// Starts multiple containers in parallel with progress bars.
///
/// Each container is spawned as a separate task. Progress is tracked via
/// `stderr` output from the container runtime. Blocks until all containers
/// have started (or failed).
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

/// Stops containers matching the server type, version, and name from CLI args.
pub fn stop_containers_by_server_type(
    server_type: ServerType,
    matches: &ArgMatches,
) -> anyhow::Result<()> {
    verify_container_command()?;
    let wildfly_containers = matches.get_one::<Vec<WildFlyContainer>>("wildfly-version");
    let name = matches.get_one::<String>("name").map(|s| s.as_str());
    block_on(async {
        let instances = container_ps(
            vec![server_type],
            wildfly_containers.map(|v| v.as_slice()),
            name,
            false,
        )
        .await?;
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
        summary("Stopped", "container", count, instant, status);
        Ok(())
    })
}

/// Stops multiple containers by their names.
pub async fn stop_containers_by_name(names: &[String]) -> anyhow::Result<()> {
    let count = names.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for name in names {
        let progress = Progress::new(name, name);
        multi_progress.add(progress.bar.clone());
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
    summary("Stopped", "container", count, instant, status);
    Ok(())
}

// ------------------------------------------------------ internal

/// Checks for name collisions against all running containers, not just wado-managed ones.
///
/// This catches conflicts with non-wado containers and race conditions that
/// [`super::resolve::resolve_start_specs`] cannot detect.
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
