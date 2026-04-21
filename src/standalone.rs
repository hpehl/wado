use crate::args::{
    extract_config, operations_argument, parameters_argument, start_spec, stop_command,
    validate_multiple_versions, versions_argument,
};
use crate::container::{
    container_network, container_run, resolve_start_specs, run_instances, verify_container_command,
};
use crate::wildfly::{ServerType, StandaloneInstance};
use clap::ArgMatches;
use futures::executor::block_on;

// ------------------------------------------------------ start

pub fn standalone_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;
    let wildfly_containers = versions_argument(matches);
    if wildfly_containers.len() > 1 {
        validate_multiple_versions(matches, &["name", "http", "management", "offset"])?;
    }
    let specs = wildfly_containers
        .iter()
        .map(|wc| start_spec(matches, wc, ServerType::Standalone))
        .collect();
    let resolved = block_on(resolve_start_specs(ServerType::Standalone, specs))?;
    let instances: Vec<StandaloneInstance> = resolved
        .into_iter()
        .map(|r| StandaloneInstance::new(r.admin_container, r.name, r.ports.unwrap()))
        .collect();
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
