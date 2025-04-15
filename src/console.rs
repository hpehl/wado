use crate::container::{container_ps, get_instance};
use crate::wildfly::ManagementClient;
use crate::wildfly::ServerType::{DomainController, Standalone};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use wildfly_container_versions::WildFlyContainer;

pub fn console(matches: &ArgMatches) -> anyhow::Result<()> {
    let management_clients = get_management_clients(matches)?;
    for client in management_clients {
        let url = format!("http://localhost:{}/console", client.management_port);
        webbrowser::open(&url)?;
    }
    Ok(())
}

fn get_management_clients(matches: &ArgMatches) -> anyhow::Result<Vec<ManagementClient>> {
    if let Some(name) = matches.get_one::<String>("name") {
        let wildfly_containers = matches.get_one::<Vec<WildFlyContainer>>("wildfly-version");
        if let Some(wildfly_containers) = wildfly_containers {
            if wildfly_containers.len() > 1 {
                bail!(
                    "Option <name> is not allowed when multiple <wildfly-version> are specified!"
                );
            }
        }
        let instance = block_on(get_instance(wildfly_containers, Some(name)))?;
        Ok(vec![ManagementClient::from_container_instance(&instance)])
    } else if let Some(wildfly_containers) =
        matches.get_one::<Vec<WildFlyContainer>>("wildfly-version")
    {
        if wildfly_containers.len() == 1 {
            Ok(vec![ManagementClient::custom_port(
                &wildfly_containers[0],
                *matches
                    .get_one::<u16>("management")
                    .unwrap_or(&(wildfly_containers[0].management_port())),
            )])
        } else {
            if matches.contains_id("name") {
                bail!(
                    "Option <name> is not allowed when multiple <wildfly-version> are specified!"
                );
            }
            if matches.contains_id("management") {
                bail!(
                    "Option <management> is not allowed when multiple <wildfly-version> are specified!"
                );
            }
            Ok(wildfly_containers
                .iter()
                .map(ManagementClient::default_port)
                .collect())
        }
    } else {
        let containers = block_on(container_ps(
            vec![Standalone, DomainController],
            None,
            None,
            true,
        ))?;
        Ok(containers
            .iter()
            .map(ManagementClient::from_container_instance)
            .collect())
    }
}
