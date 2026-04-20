use crate::args::{
    name_argument, operations_argument, parameters_argument, port_argument, server_argument,
    stop_command, validate_single_version, versions_argument,
};
use crate::constants::{HOSTNAME_VARIABLE, WILDFLY_ADMIN_CONTAINER};
use crate::container::{
    add_servers, container_network, container_run, ensure_unique_names, run_instances,
    running_counts, running_instance_count, verify_container_command,
};
use crate::wildfly::{AdminContainer, DomainController, Ports, Server, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn dc_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let has_custom_name = matches.get_one::<String>("name").is_some();
    let has_custom_ports = matches.get_one::<u16>("http").is_some()
        || matches.get_one::<u16>("management").is_some()
        || matches.get_one::<u16>("offset").is_some();
    let instances = if wildfly_containers.len() == 1 {
        let wildfly_container = wildfly_containers[0].clone();
        let admin_container =
            AdminContainer::new(wildfly_container.clone(), ServerType::DomainController);
        let mut instance = DomainController::new(
            admin_container.clone(),
            name_argument("name", matches, || admin_container.container_name()),
            port_argument(matches, &wildfly_container),
        );
        if !has_custom_name && !has_custom_ports {
            let count = block_on(running_instance_count(
                ServerType::DomainController,
                &wildfly_container,
            ))?;
            if count > 0 {
                instance = instance.copy(count);
            }
        }
        vec![instance]
    } else {
        validate_single_version(matches, &["name", "http", "management", "offset"])?;
        let instances = wildfly_containers
            .iter()
            .map(|wildfly_container| {
                let admin_container =
                    AdminContainer::new(wildfly_container.clone(), ServerType::DomainController);
                DomainController::new(
                    admin_container.clone(),
                    admin_container.container_name(),
                    Ports::default_ports(wildfly_container),
                )
            })
            .collect::<Vec<_>>();
        let running_counts = block_on(running_counts(
            ServerType::DomainController,
            &wildfly_containers,
        ))?;
        ensure_unique_names(&instances, DomainController::copy, |wc| {
            *running_counts.get(&wc.identifier).unwrap_or(&0)
        })
    };
    block_on(start_instances(
        instances,
        server_argument(matches),
        operations_argument(matches),
        parameters_argument(matches),
    ))
}

async fn start_instances(
    instances: Vec<DomainController>,
    servers: Vec<Server>,
    operations: Vec<String>,
    parameters: Vec<String>,
) -> anyhow::Result<()> {
    container_network().await?;
    run_instances(&instances, |instance| {
        let mut command = container_run(
            &instance.name,
            Some(&instance.ports),
            operations.clone(),
            instance.admin_container.wildfly_container.is_dev(),
            None,
        );
        command
            .arg("--network")
            .arg(WILDFLY_ADMIN_CONTAINER)
            .arg("--env")
            .arg(format!("{}={}", HOSTNAME_VARIABLE, instance.name));
        let mut command = add_servers(command, &instance.name, servers.clone());
        command
            .arg(instance.admin_container.image_name())
            .args(parameters.clone());
        command
    })
    .await
}

// ------------------------------------------------------ stop

pub fn dc_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    stop_command(ServerType::DomainController, matches)
}
