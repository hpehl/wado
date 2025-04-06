use crate::args::{
    name_argument, operations_argument, parameters_argument, port_argument, versions_argument,
};
use crate::command::summary;
use crate::podman::{create_network, podman_run, verify_podman};
use crate::progress::{Progress, stderr_reader};
use crate::wildfly::{
    AdminContainer, Ports, ServerType, StandaloneInstance, ensure_unique_names, stop_instances,
};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_podman()?;

    let wildfly_containers = versions_argument(matches);
    let instances = if wildfly_containers.len() == 1 {
        let wildfly_container = wildfly_containers[0].clone();
        let admin_container = AdminContainer::standalone(wildfly_container.clone());
        vec![StandaloneInstance::new(
            admin_container.clone(),
            name_argument("name", matches, || admin_container.container_name()),
            port_argument(matches, &wildfly_container),
        )]
    } else {
        if matches.contains_id("name") {
            bail!("Option <name> is not allowed when multiple <wildfly-version> are specified!");
        }
        if matches.contains_id("http") {
            bail!("Option <http> is not allowed when multiple <wildfly-version> are specified!");
        }
        if matches.contains_id("management") {
            bail!(
                "Option <management> is not allowed when multiple <wildfly-version> are specified!"
            );
        }
        if matches.contains_id("offset") {
            bail!("Option <offset> is not allowed when multiple <wildfly-version> are specified!");
        }
        let instances = wildfly_containers
            .iter()
            .map(|wildfly_container| {
                let admin_container = AdminContainer::standalone(wildfly_container.clone());
                StandaloneInstance::new(
                    admin_container.clone(),
                    admin_container.container_name(),
                    Ports::default_ports(wildfly_container),
                )
            })
            .collect::<Vec<_>>();
        ensure_unique_names(&instances, StandaloneInstance::copy)
    };
    block_on(start_instances(
        instances,
        parameters_argument(matches),
        operations_argument(matches),
    ))
}

async fn start_instances(
    instances: Vec<StandaloneInstance>,
    parameters: Vec<String>,
    operations: Vec<String>,
) -> anyhow::Result<()> {
    let count = instances.len();
    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    create_network().await?;
    for instance in instances {
        let progress = Progress::new(
            &instance.admin_container.wildfly_container.short_version,
            &instance.admin_container.image_name(),
        );
        multi_progress.add(progress.bar.clone());
        let mut command = podman_run(
            instance.name.as_str(),
            Some(&instance.ports),
            operations.clone(),
        );
        command
            .arg(instance.admin_container.image_name())
            .args(parameters.clone());
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-run.");

        let stderr = stderr_reader(&mut child);
        let progress_clone = progress.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish(output, Some(&instance.name))
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stderr).await;
        });
    }

    let status = commands.join_all().await;
    summary("Started", "container", count, instant, status);
    Ok(())
}

// ------------------------------------------------------ stop

pub fn standalone_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_podman()?;
    let wildfly_containers = matches.get_one::<Vec<WildFlyContainer>>("wildfly-version");
    let name = matches.get_one::<String>("name").map(|s| s.as_str());
    block_on(stop_instances(
        ServerType::Standalone,
        wildfly_containers,
        name,
    ))
}
