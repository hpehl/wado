use crate::args::{
    extract_config, name_argument, operations_argument, parameters_argument, server_argument,
    stop_command, username_password_argument, versions_argument,
};
use crate::constants::{
    DOMAIN_CONTROLLER_VARIABLE, HOSTNAME_VARIABLE, PASSWORD_VARIABLE, USERNAME_VARIABLE,
    WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{
    add_servers, container_command, container_network, container_run, resolve_start_specs,
    run_instances, verify_container_command,
};
use crate::wildfly::{AdminContainer, HostController, Server, ServerType, StartSpec};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use std::process::Stdio;
use tokio::try_join;
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ start

pub fn hc_start(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let wildfly_containers = versions_argument(matches);
    let wildfly_container = wildfly_containers[0].clone();
    let admin_container_dc =
        AdminContainer::new(wildfly_container.clone(), ServerType::DomainController);
    let dc_name = name_argument("domain-controller", matches, || {
        admin_container_dc.container_name()
    });

    if wildfly_containers.len() > 1 {
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
    }

    let specs: Vec<StartSpec> = wildfly_containers
        .iter()
        .map(|wc| StartSpec {
            admin_container: AdminContainer::new(wc.clone(), ServerType::HostController),
            custom_name: matches.get_one::<String>("name").cloned(),
            custom_http: None,
            custom_management: None,
        })
        .collect();
    let resolved = block_on(resolve_start_specs(ServerType::HostController, specs))?;
    let instances: Vec<HostController> = resolved
        .into_iter()
        .map(|r| HostController::new(r.admin_container, r.name, dc_name.clone()))
        .collect();

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
    let config = extract_config(&parameters, "domain.xml");
    run_instances(&instances, |instance| {
        let mut command = container_run(
            &instance.name,
            None,
            operations.clone(),
            instance.admin_container.wildfly_container.is_dev(),
            None,
            Some(&config),
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
        let mut command = add_servers(command, &instance.name, servers.clone());
        command
            .arg(instance.admin_container.image_name())
            .args(parameters.clone());
        command
    })
    .await
}

pub(crate) async fn create_secret(secret_name: &str, secret_value: &str) -> anyhow::Result<()> {
    let mut podman_secret = container_command()?
        .arg("secret")
        .arg("create")
        .arg("--replace")
        .arg(secret_name)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn podman secret");
    if let Some(mut stdin) = podman_secret.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(secret_value.as_bytes()).await?;
    }
    podman_secret.wait().await?;
    Ok(())
}

// ------------------------------------------------------ stop

pub fn hc_stop(matches: &ArgMatches) -> anyhow::Result<()> {
    stop_command(ServerType::HostController, matches)
}
