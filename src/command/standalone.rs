use super::lifecycle::{prepare_instances, run_instances, stop_containers_by_server_type};
use crate::args::{extract_config, operations_argument, parameters_argument};
use crate::container::{container_network_cmd, container_run_cmd};
use crate::wildfly::{ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    let instances: Vec<StandaloneInstance> = prepare_instances(
        matches,
        ServerType::Standalone,
        &["name", "http", "management", "offset"],
        |r| StandaloneInstance::new(r.admin_container, r.name, r.ports.unwrap()),
    )?;
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
    container_network_cmd().await?;
    run_instances(&instances, |instance| {
        let mut command = container_run_cmd(
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
    stop_containers_by_server_type(ServerType::Standalone, matches)
}
