use crate::args::{
    name_argument, operations_argument, parameters_argument, port_argument, stop_command,
    validate_single_version, versions_argument,
};
use crate::container::{
    container_network, container_run, ensure_unique_names, run_instances, verify_container_command,
};
use crate::wildfly::{AdminContainer, Ports, ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let instances = if wildfly_containers.len() == 1 {
        let wildfly_container = wildfly_containers[0].clone();
        let admin_container =
            AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
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
                let admin_container =
                    AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
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
    container_network().await?;
    run_instances(&instances, |instance| {
        let mut command = container_run(&instance.name, Some(&instance.ports), operations.clone());
        command
            .arg(instance.admin_container.image_name())
            .args(parameters.clone());
        command
    })
    .await
}

// ------------------------------------------------------ stop

pub fn standalone_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    stop_command(ServerType::Standalone, matches)
}
