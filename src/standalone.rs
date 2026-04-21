use crate::args::{
    extract_config, name_argument, operations_argument, parameters_argument, port_argument,
    stop_command, validate_single_version, versions_argument,
};
use crate::container::{
    container_network, container_run, ensure_unique_instances, run_instances, running_counts,
    running_counts_by_type, running_instance_count, running_instance_count_by_type,
    verify_container_command,
};
use crate::wildfly::{AdminContainer, Ports, ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let has_custom_name = matches.get_one::<String>("name").is_some();
    let has_custom_ports = matches.get_one::<u16>("http").is_some()
        || matches.get_one::<u16>("management").is_some()
        || matches.get_one::<u16>("offset").is_some();
    let instances = if wildfly_containers.len() == 1 {
        let wildfly_container = wildfly_containers[0].clone();
        let admin_container =
            AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
        let mut instance = StandaloneInstance::new(
            admin_container.clone(),
            name_argument("name", matches, || admin_container.container_name()),
            port_argument(matches, &wildfly_container),
        );
        if !has_custom_name && !has_custom_ports {
            let same_type = block_on(running_instance_count_by_type(
                ServerType::Standalone,
                &wildfly_container,
            ))?;
            let all_types = block_on(running_instance_count(&wildfly_container))?;
            if same_type > 0 || all_types > 0 {
                let name_index = if same_type > 0 {
                    Some(same_type)
                } else {
                    None
                };
                instance = instance.copy(name_index, all_types);
            }
        }
        vec![instance]
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
        let same_type_counts = block_on(running_counts_by_type(
            ServerType::Standalone,
            &wildfly_containers,
        ))?;
        let all_type_counts = block_on(running_counts(&wildfly_containers))?;
        ensure_unique_instances(
            &instances,
            StandaloneInstance::copy,
            |wc| *same_type_counts.get(&wc.identifier).unwrap_or(&0),
            |wc| *all_type_counts.get(&wc.identifier).unwrap_or(&0),
        )
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
    let config = extract_config(&parameters, "standalone.xml");
    container_network().await?;
    run_instances(&instances, |instance| {
        let mut command = container_run(
            &instance.name,
            Some(&instance.ports),
            operations.clone(),
            instance.admin_container.wildfly_container.is_dev(),
            None,
            Some(&config),
        );
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
