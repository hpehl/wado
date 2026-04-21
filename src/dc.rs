use crate::args::{
    extract_config, operations_argument, parameters_argument, server_argument, start_spec,
    stop_command, validate_single_version, versions_argument,
};
use crate::constants::{HOSTNAME_VARIABLE, WILDFLY_ADMIN_CONTAINER};
use crate::container::{
    add_servers, container_network, container_run, resolve_start_specs, run_instances,
    verify_container_command,
};
use crate::wildfly::{DomainController, Server, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn dc_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;
    let wildfly_containers = versions_argument(matches);
    if wildfly_containers.len() > 1 {
        validate_single_version(matches, &["name", "http", "management", "offset"])?;
    }
    let specs = wildfly_containers
        .iter()
        .map(|wc| start_spec(matches, wc, ServerType::DomainController))
        .collect();
    let resolved = block_on(resolve_start_specs(ServerType::DomainController, specs))?;
    let instances: Vec<DomainController> = resolved
        .into_iter()
        .map(|r| DomainController::new(r.admin_container, r.name, r.ports.unwrap()))
        .collect();
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
