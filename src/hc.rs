use crate::args::{
    name_argument, operations_argument, parameters_argument, server_argument, stop_command,
    username_password_argument, versions_argument,
};
use crate::constants::{
    DOMAIN_CONTROLLER_VARIABLE, HOSTNAME_VARIABLE, PASSWORD_VARIABLE, USERNAME_VARIABLE,
    WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{
    add_servers, container_command, container_network, container_run, ensure_unique_names,
    run_instances, verify_container_command,
};
use crate::wildfly::{AdminContainer, HostController, Server, ServerType};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use std::process::Stdio;
use tokio::process::Command;
use tokio::{join, try_join};
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ start

pub fn hc_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let wildfly_container = wildfly_containers[0].clone();
    let admin_container_dc = AdminContainer::new(wildfly_container.clone(), ServerType::DomainController);
    let dc_name = name_argument("domain-controller", matches, || {
        admin_container_dc.container_name()
    });
    let instances = if wildfly_containers.len() == 1 {
        let admin_container_hc = AdminContainer::new(wildfly_container.clone(), ServerType::HostController);
        vec![HostController::new(
            admin_container_hc.clone(),
            name_argument("name", matches, || admin_container_hc.container_name()),
            dc_name.to_string(),
        )]
    } else {
        if matches.contains_id("name") {
            bail!("Option <name> is not allowed when multiple <wildfly-version> are specified!");
        }
        if !same_versions(wildfly_containers.as_slice())
            && !matches.contains_id("domain-controller")
        {
            bail!(
                "Option <domain-controller> is required when multiple <wildfly-version> are specified!"
            );
        }
        let instances = wildfly_containers
            .iter()
            .map(|wildfly_container| {
                let admin_container = AdminContainer::new(wildfly_container.clone(), ServerType::HostController);
                HostController::new(
                    admin_container.clone(),
                    admin_container.container_name(),
                    dc_name.to_string(),
                )
            })
            .collect::<Vec<_>>();
        ensure_unique_names(&instances, HostController::copy)
    };
    let (username, password) = username_password_argument(matches);
    let mut parameters = parameters_argument(matches);
    let primary_address = format!("--primary-address={}", dc_name);
    parameters.push(primary_address);
    block_on(start_instances(
        instances,
        username,
        password,
        server_argument(matches),
        operations_argument(matches),
        parameters,
    ))
}

fn same_versions(instances: &[WildFlyContainer]) -> bool {
    instances
        .iter()
        .map(|c| c.identifier)
        .all(|identifier| identifier == instances[0].identifier)
}

async fn start_instances(
    instances: Vec<HostController>,
    username: &str,
    password: &str,
    servers: Vec<Server>,
    operations: Vec<String>,
    parameters: Vec<String>,
) -> anyhow::Result<()> {
    try_join!(
        container_network(),
        create_secret("username", username),
        create_secret("password", password)
    )?;
    run_instances(&instances, |instance| {
        let mut command = container_run(&instance.name, None, operations.clone());
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
        let mut command = add_servers(command, &instance.name, servers.clone());
        command
            .arg(instance.admin_container.image_name())
            .args(parameters.clone());
        command
    })
    .await
}

async fn create_secret(secret_name: &str, secret_value: &str) -> anyhow::Result<()> {
    let mut echo = Command::new("echo")
        .arg("-n")
        .arg(secret_value)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn echo");
    let podman_secret_stdin: Stdio = echo
        .stdout
        .take()
        .unwrap()
        .try_into()
        .expect("Failed to convert to stdio");
    let mut podman_secret = container_command()?
        .arg("secret")
        .arg("create")
        .arg("--replace")
        .arg(secret_name)
        .arg("-")
        .stdin(podman_secret_stdin)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn podman secret");
    let (echo_result, podman_secret_result) = join!(echo.wait(), podman_secret.wait());
    echo_result?;
    podman_secret_result?;
    Ok(())
}

// ------------------------------------------------------ stop

pub fn hc_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    stop_command(ServerType::HostController, matches)
}
