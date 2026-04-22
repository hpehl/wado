use crate::args::{
    extract_config, operations_argument, parameters_argument, resolve_instances, server_argument,
    stop_command,
};
use crate::constants::{HOSTNAME_VARIABLE, WILDFLY_ADMIN_CONTAINER};
use crate::container::{
    add_servers, container_network_cmd, container_run_cmd, run_instances,
};
use crate::wildfly::{DomainController, Server, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn dc_start(matches: &ArgMatches) -> anyhow::Result<()> {
    let instances: Vec<DomainController> = resolve_instances(
        matches,
        ServerType::DomainController,
        &["name", "http", "management", "offset"],
        |r| DomainController::new(r.admin_container, r.name, r.ports.unwrap()),
    )?;
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
    let config = extract_config(&parameters, "domain.xml");
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
