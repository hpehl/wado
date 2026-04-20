use crate::constants::{
    DOMAIN_CONTROLLER_VARIABLE, HOSTNAME_VARIABLE, PASSWORD_VARIABLE, USERNAME_VARIABLE,
    WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{
    add_servers, container_network, container_run, run_instances, stop_containers_by_name,
    verify_container_command,
};
use crate::hc::create_secret;
use crate::topology_model::TopologySetup;
use crate::wildfly::{AdminContainer, DomainController, HostController, Ports, Server, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::try_join;
use wildfly_container_versions::WildFlyContainer;

pub fn topology_start(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.get_one::<PathBuf>("setup").unwrap();
    let setup = TopologySetup::load(path)?;
    verify_container_command()?;

    let dc_host = setup.dc_host();
    let dc_version = dc_host.effective_version(setup.version);
    let dc_wf =
        WildFlyContainer::version(&dc_version.to_string()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let dc_admin = AdminContainer::new(dc_wf.clone(), ServerType::DomainController);
    let dc = DomainController::new(dc_admin, dc_host.name.clone(), Ports::default_ports(&dc_wf));
    let dc_servers: Vec<Server> = dc_host.servers.iter().map(|s| s.to_server()).collect();

    let mut hc_server_map: HashMap<String, Vec<Server>> = HashMap::new();
    let hcs: Vec<HostController> = setup
        .hc_hosts()
        .iter()
        .map(|host| {
            let version = host.effective_version(setup.version);
            let wf = WildFlyContainer::version(&version.to_string())
                .map_err(|e| anyhow::anyhow!("{}", e))
                .unwrap();
            let admin = AdminContainer::new(wf, ServerType::HostController);
            let servers: Vec<Server> = host.servers.iter().map(|s| s.to_server()).collect();
            hc_server_map.insert(host.name.clone(), servers);
            HostController::new(admin, host.name.clone(), dc_host.name.clone())
        })
        .collect();

    block_on(start_topology(dc, dc_servers, hcs, hc_server_map))
}

async fn start_topology(
    dc: DomainController,
    dc_servers: Vec<Server>,
    hcs: Vec<HostController>,
    hc_server_map: HashMap<String, Vec<Server>>,
) -> anyhow::Result<()> {
    try_join!(
        container_network(),
        create_secret("username", "admin"),
        create_secret("password", "admin"),
    )?;

    run_instances(std::slice::from_ref(&dc), |instance| {
        let mut command = container_run(&instance.name, Some(&instance.ports), vec![], false);
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
            let mut command = container_run(&instance.name, None, vec![], false);
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
    let path = matches.get_one::<PathBuf>("setup").unwrap();
    let setup = TopologySetup::load(path)?;
    verify_container_command()?;

    let dc_name = setup.dc_host().name.clone();
    let hc_names: Vec<String> = setup.hc_hosts().iter().map(|h| h.name.clone()).collect();

    block_on(stop_topology(hc_names, dc_name))
}

async fn stop_topology(hc_names: Vec<String>, dc_name: String) -> anyhow::Result<()> {
    if !hc_names.is_empty() {
        stop_containers_by_name(&hc_names).await?;
    }
    stop_containers_by_name(&[dc_name]).await?;
    Ok(())
}
