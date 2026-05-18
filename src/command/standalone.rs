use super::lifecycle::{
    apply_ports, prepare_instances, print_json_results, run_instances,
    stop_containers_by_server_type, wait_for_instances,
};
use crate::args::{extract_config, operations_argument, parameters_argument};
use crate::container::{container_network_cmd, container_run_cmd};
use crate::wildfly::{ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;
use wildfly_meta::WildFlyImageRegistry;

// ------------------------------------------------------ start

pub fn standalone_start(
    matches: &ArgMatches,
    registry: &WildFlyImageRegistry,
    json: bool,
) -> anyhow::Result<()> {
    let instances: Vec<StandaloneInstance> = prepare_instances(
        matches,
        ServerType::Standalone,
        &["name", "http", "management", "offset"],
        |r| StandaloneInstance::new(r.admin_image, r.name, r.ports.unwrap()),
        registry,
    )?;
    block_on(start_instances(
        instances,
        parameters_argument(matches),
        operations_argument(matches),
        json,
    ))
}

async fn start_instances(
    instances: Vec<StandaloneInstance>,
    parameters: Vec<String>,
    operations: Vec<String>,
    json: bool,
) -> anyhow::Result<()> {
    let config = extract_config(&parameters, "standalone.xml");
    container_network_cmd().await?;

    let port_map: Vec<(String, u16, u16)> = instances
        .iter()
        .map(|i| (i.name.clone(), i.ports.http, i.ports.management))
        .collect();

    let (status, _instant) = run_instances(
        &instances,
        |instance| {
            let mut command = container_run_cmd(
                &instance.name,
                Some(&instance.ports),
                operations.clone(),
                instance.admin_image.wildfly_image.is_dev(),
                None,
                Some(&config),
            );
            command
                .arg(instance.admin_image.image_name())
                .args(parameters.clone());
            command
        },
        json,
    )
    .await?;

    let mut status = apply_ports(status, &port_map);
    wait_for_instances(&mut status, json).await;

    if json {
        print_json_results(&status);
    }
    Ok(())
}

// ------------------------------------------------------ stop

pub fn standalone_stop(
    matches: &ArgMatches,
    registry: &WildFlyImageRegistry,
    json: bool,
) -> anyhow::Result<()> {
    let status = stop_containers_by_server_type(ServerType::Standalone, matches, registry, json)?;
    if json {
        print_json_results(&status);
    }
    Ok(())
}
