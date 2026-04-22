use crate::command::lifecycle::stop_containers_by_name;
use crate::container::{containers_by_topology, verify_container_command};
use crate::wildfly::ServerType;
use clap::ArgMatches;
use futures::executor::block_on;

use super::model::TopologySetup;

pub fn topology_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    let setup_arg = matches.get_one::<String>("setup").unwrap();
    let topology_name = resolve_topology_name(setup_arg)?;
    verify_container_command()?;
    block_on(stop_topology(&topology_name))
}

fn resolve_topology_name(setup_arg: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(setup_arg);
    if path.exists() {
        let setup = TopologySetup::load(path)?;
        Ok(setup.name)
    } else {
        Ok(setup_arg.to_string())
    }
}

async fn stop_topology(topology_name: &str) -> anyhow::Result<()> {
    let instances = containers_by_topology(topology_name).await?;
    if instances.is_empty() {
        println!(
            "No running containers found for topology '{}'",
            topology_name
        );
        return Ok(());
    }

    let hc_names: Vec<String> = instances
        .iter()
        .filter(|i| i.admin_container.server_type == ServerType::HostController)
        .map(|i| i.name.clone())
        .collect();
    let dc_names: Vec<String> = instances
        .iter()
        .filter(|i| i.admin_container.server_type == ServerType::DomainController)
        .map(|i| i.name.clone())
        .collect();

    if !hc_names.is_empty() {
        stop_containers_by_name(&hc_names).await?;
    }
    if !dc_names.is_empty() {
        stop_containers_by_name(&dc_names).await?;
    }
    Ok(())
}
