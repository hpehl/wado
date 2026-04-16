use crate::args::{
    name_argument, operations_argument, parameters_argument, port_argument, stop_command,
    validate_single_version, versions_argument,
};
use crate::container::{
    container_network, container_run, ensure_unique_names, verify_container_command,
};
use crate::progress::{Progress, stderr_reader, summary};
use crate::wildfly::{AdminContainer, Ports, ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let instances = if wildfly_containers.len() == 1 {
        let wildfly_container = wildfly_containers[0].clone();
        let admin_container = AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
        vec![StandaloneInstance::new(
            admin_container.clone(),
            name_argument("name", matches, || admin_container.container_name()),
            port_argument(matches, &wildfly_container),
        )]
    } else {
        validate_single_version(matches, &["name", "http", "management", "offset"])?;
        let instances = wildfly_containers
            .iter()
            .map(|wildfly_container| {
                let admin_container = AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
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

    container_network().await?;
    for instance in instances {
        let progress = Progress::new(
            &instance.admin_container.wildfly_container.short_version,
            &instance.admin_container.image_name(),
        );
        multi_progress.add(progress.bar.clone());
        let mut command = container_run(
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
    stop_command(ServerType::Standalone, matches)
}
