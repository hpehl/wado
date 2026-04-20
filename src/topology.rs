use crate::constants::{
    DOMAIN_CONTROLLER_VARIABLE, HOSTNAME_VARIABLE, PASSWORD_VARIABLE, USERNAME_VARIABLE,
    WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{
    add_servers, container_network, container_run, containers_by_topology, ensure_unique_names,
    run_instances, running_counts, running_instance_count, stop_containers_by_name,
    verify_container_command,
};
use crate::hc::create_secret;
use crate::topology_model::TopologySetup;
use crate::wildfly::{AdminContainer, DomainController, HostController, Ports, Server, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::try_join;
use wildfly_container_versions::WildFlyContainer;

pub fn topology_start(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.get_one::<PathBuf>("setup").unwrap();
    let setup = TopologySetup::load(path)?;
    verify_container_command()?;

    let topology_name = setup.name.clone();

    let dc_host = setup.dc_host();
    let dc_version = dc_host.effective_version(setup.version);
    let dc_wf =
        WildFlyContainer::version(&dc_version.to_string()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let dc_admin = AdminContainer::new(dc_wf.clone(), ServerType::DomainController);
    let dc_name = dc_host
        .name
        .clone()
        .unwrap_or_else(|| dc_admin.container_name());
    let mut dc = DomainController::new(dc_admin, dc_name, Ports::default_ports(&dc_wf));
    if dc_host.name.is_none() {
        let count = block_on(running_instance_count(ServerType::DomainController, &dc_wf))?;
        if count > 0 {
            dc = dc.copy(count);
        }
    }
    let dc_servers: Vec<Server> = dc_host.servers.iter().map(|s| s.to_server()).collect();

    let hc_hosts = setup.hc_hosts();
    let (named_hcs, unnamed_hcs) = build_host_controllers(&hc_hosts, setup.version, &dc.name)?;

    let unnamed_hcs = if !unnamed_hcs.is_empty() {
        let wf_containers: Vec<WildFlyContainer> = unnamed_hcs
            .iter()
            .map(|hc| hc.admin_container.wildfly_container.clone())
            .collect();
        let counts = block_on(running_counts(ServerType::HostController, &wf_containers))?;
        ensure_unique_names(&unnamed_hcs, HostController::copy, |wc| {
            *counts.get(&wc.identifier).unwrap_or(&0)
        })
    } else {
        unnamed_hcs
    };

    let hc_server_map = build_server_map(&hc_hosts, &named_hcs, &unnamed_hcs);

    let mut all_hcs = named_hcs;
    all_hcs.extend(unnamed_hcs);

    block_on(start_topology(
        topology_name,
        dc,
        dc_servers,
        all_hcs,
        hc_server_map,
    ))
}

fn build_host_controllers(
    hc_hosts: &[&crate::topology_model::HostSetup],
    default_version: u16,
    dc_name: &str,
) -> anyhow::Result<(Vec<HostController>, Vec<HostController>)> {
    let mut named = Vec::new();
    let mut unnamed = Vec::new();

    for host in hc_hosts {
        let version = host.effective_version(default_version);
        let wf = WildFlyContainer::version(&version.to_string())
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let admin = AdminContainer::new(wf, ServerType::HostController);

        if let Some(name) = &host.name {
            named.push(HostController::new(
                admin,
                name.clone(),
                dc_name.to_string(),
            ));
        } else {
            let default_name = admin.container_name();
            unnamed.push(HostController::new(
                admin,
                default_name,
                dc_name.to_string(),
            ));
        }
    }

    Ok((named, unnamed))
}

fn build_server_map(
    hc_hosts: &[&crate::topology_model::HostSetup],
    named_hcs: &[HostController],
    unnamed_hcs: &[HostController],
) -> BTreeMap<String, Vec<Server>> {
    let mut map = BTreeMap::new();
    let mut unnamed_idx = 0;
    for host in hc_hosts {
        let servers: Vec<Server> = host.servers.iter().map(|s| s.to_server()).collect();
        let resolved_name = if let Some(name) = &host.name {
            named_hcs
                .iter()
                .find(|hc| hc.name == name.as_str())
                .map(|hc| hc.name.clone())
        } else {
            let name = unnamed_hcs.get(unnamed_idx).map(|hc| hc.name.clone());
            unnamed_idx += 1;
            name
        };
        if let Some(name) = resolved_name
            && !servers.is_empty()
        {
            map.insert(name, servers);
        }
    }
    map
}

async fn start_topology(
    topology_name: String,
    dc: DomainController,
    dc_servers: Vec<Server>,
    hcs: Vec<HostController>,
    hc_server_map: BTreeMap<String, Vec<Server>>,
) -> anyhow::Result<()> {
    try_join!(
        container_network(),
        create_secret("username", "admin"),
        create_secret("password", "admin"),
    )?;

    let topology = topology_name.as_str();

    run_instances(std::slice::from_ref(&dc), |instance| {
        let mut command = container_run(
            &instance.name,
            Some(&instance.ports),
            vec![],
            false,
            Some(topology),
        );
        command
            .arg("--network")
            .arg(WILDFLY_ADMIN_CONTAINER)
            .arg("--env")
            .arg(format!("{}={}", HOSTNAME_VARIABLE, instance.name));
        let mut command = add_servers(command, &instance.name, dc_servers.clone());
        command.arg(instance.admin_container.image_name());
        command
    })
    .await?;

    if !hcs.is_empty() {
        run_instances(&hcs, |instance| {
            let servers = hc_server_map
                .get(&instance.name)
                .cloned()
                .unwrap_or_default();
            let mut command = container_run(&instance.name, None, vec![], false, Some(topology));
            command
                .arg(format!(
                    "--secret=username,type=env,target={}",
                    USERNAME_VARIABLE
                ))
                .arg(format!(
                    "--secret=password,type=env,target={}",
                    PASSWORD_VARIABLE
                ))
                .arg("--network")
                .arg(WILDFLY_ADMIN_CONTAINER)
                .arg("--env")
                .arg(format!("{}={}", HOSTNAME_VARIABLE, instance.name))
                .arg("--env")
                .arg(format!(
                    "{}={}",
                    DOMAIN_CONTROLLER_VARIABLE, instance.domain_controller
                ));
            let mut command = add_servers(command, &instance.name, servers);
            command
                .arg(instance.admin_container.image_name())
                .arg(format!("--primary-address={}", instance.domain_controller));
            command
        })
        .await?;
    }

    Ok(())
}

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
