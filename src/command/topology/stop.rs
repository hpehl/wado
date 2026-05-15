use crate::command::lifecycle::{print_json_results, stop_containers_by_name};
use crate::container::{containers_by_topology, verify_container_command};
use crate::wildfly::ServerType;
use clap::ArgMatches;
use futures::executor::block_on;
use wildfly_meta::WildFlyImageRegistry;

use super::model::TopologySetup;

pub fn topology_stop(
    matches: &ArgMatches,
    registry: &WildFlyImageRegistry,
    json: bool,
) -> anyhow::Result<()> {
    let setup_arg = matches.get_one::<String>("setup").unwrap();
    let topology_name = resolve_topology_name(setup_arg, registry)?;
    verify_container_command()?;
    block_on(stop_topology(&topology_name, registry, json))
}

fn resolve_topology_name(
    setup_arg: &str,
    registry: &WildFlyImageRegistry,
) -> anyhow::Result<String> {
    let path = std::path::Path::new(setup_arg);
    if path.exists() {
        let setup = TopologySetup::load(path, registry)?;
        Ok(setup.name)
    } else {
        Ok(setup_arg.to_string())
    }
}

async fn stop_topology(
    topology_name: &str,
    registry: &WildFlyImageRegistry,
    json: bool,
) -> anyhow::Result<()> {
    let instances = containers_by_topology(topology_name, registry).await?;
    if instances.is_empty() {
        if json {
            println!("[]");
        } else {
            println!(
                "No running containers found for topology '{}'",
                topology_name
            );
        }
        return Ok(());
    }

    let hc_names: Vec<String> = instances
        .iter()
        .filter(|i| i.admin_image.server_type == ServerType::HostController)
        .map(|i| i.name.clone())
        .collect();
    let dc_names: Vec<String> = instances
        .iter()
        .filter(|i| i.admin_image.server_type == ServerType::DomainController)
        .map(|i| i.name.clone())
        .collect();

    let mut all_status = vec![];

    if !hc_names.is_empty() {
        let status = stop_containers_by_name(&hc_names, json).await?;
        all_status.extend(status);
    }
    if !dc_names.is_empty() {
        let status = stop_containers_by_name(&dc_names, json).await?;
        all_status.extend(status);
    }

    if json {
        print_json_results(&all_status);
    }
    Ok(())
}
