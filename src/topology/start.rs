use crate::constants::{
    DOMAIN_CONTROLLER_VARIABLE, HOSTNAME_VARIABLE, PASSWORD_VARIABLE, USERNAME_VARIABLE,
    WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{
    add_servers, container_network_cmd, container_run_cmd, create_secret, resolve_start_specs,
    run_instances, verify_container_command,
};
use crate::wildfly::{
    AdminContainer, DEFAULT_SERVER_OFFSET, DomainController, HostController, Server, ServerType,
    StartSpec, apply_offsets,
};
use clap::ArgMatches;
use futures::executor::block_on;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::try_join;
use wildfly_container_versions::WildFlyContainer;

use super::model::{HostSetup, TopologySetup};

pub fn topology_start(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.get_one::<PathBuf>("setup").unwrap();
    let setup = TopologySetup::load(path)?;
    verify_container_command()?;

    let topology_name = setup.name.clone();

    let dc_host = setup.dc_host();
    let dc_version = dc_host.effective_version(setup.version);
    let dc_wf =
        WildFlyContainer::version(&dc_version.to_string()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let dc_spec = StartSpec {
        admin_container: AdminContainer::new(dc_wf, ServerType::DomainController),
        custom_name: dc_host.name.clone(),
        custom_http: None,
        custom_management: None,
    };
    let dc_resolved = block_on(resolve_start_specs(
        ServerType::DomainController,
        vec![dc_spec],
    ))?;
    let dc_r = &dc_resolved[0];
    let dc = DomainController::new(
        dc_r.admin_container.clone(),
        dc_r.name.clone(),
        dc_r.ports.clone().unwrap(),
    );
    let dc_servers: Vec<Server> = dc_host.servers.iter().map(|s| s.to_server()).collect();
    let dc_servers = apply_offsets(dc_servers, DEFAULT_SERVER_OFFSET);

    let hc_hosts = setup.hc_hosts();
    let hc_specs = build_hc_specs(&hc_hosts, setup.version)?;
    let hc_resolved = block_on(resolve_start_specs(ServerType::HostController, hc_specs))?;
    let hcs: Vec<HostController> = hc_resolved
        .into_iter()
        .map(|r| HostController::new(r.admin_container, r.name, dc.name.clone()))
        .collect();

    let hc_server_map = build_server_map(&hc_hosts, &hcs);

    block_on(start_topology(
        topology_name,
        dc,
        dc_servers,
        hcs,
        hc_server_map,
    ))
}

fn build_hc_specs(
    hc_hosts: &[&HostSetup],
    default_version: u16,
) -> anyhow::Result<Vec<StartSpec>> {
    hc_hosts
        .iter()
        .map(|host| {
            let version = host.effective_version(default_version);
            let wf = WildFlyContainer::version(&version.to_string())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(StartSpec {
                admin_container: AdminContainer::new(wf, ServerType::HostController),
                custom_name: host.name.clone(),
                custom_http: None,
                custom_management: None,
            })
        })
        .collect()
}

fn build_server_map(
    hc_hosts: &[&HostSetup],
    hcs: &[HostController],
) -> BTreeMap<String, Vec<Server>> {
    let mut map = BTreeMap::new();
    for (host, hc) in hc_hosts.iter().zip(hcs.iter()) {
        let servers: Vec<Server> = host.servers.iter().map(|s| s.to_server()).collect();
        let servers = apply_offsets(servers, DEFAULT_SERVER_OFFSET);
        if !servers.is_empty() {
            map.insert(hc.name.clone(), servers);
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
        container_network_cmd(),
        create_secret("username", "admin"),
        create_secret("password", "admin"),
    )?;

    let topology = topology_name.as_str();

    run_instances(std::slice::from_ref(&dc), |instance| {
        let mut command = container_run_cmd(
            &instance.name,
            Some(&instance.ports),
            vec![],
            false,
            Some(topology),
            Some("domain.xml"),
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
            let mut command = container_run_cmd(
                &instance.name,
                None,
                vec![],
                false,
                Some(topology),
                Some("domain.xml"),
            );
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
