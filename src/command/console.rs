use crate::container::{container_ps, get_instance};
use crate::wildfly::ManagementClient;
use crate::wildfly::ServerType::{DomainController, Standalone};
use anyhow::bail;
use clap::ArgMatches;
use futures::executor::block_on;
use wildfly_meta::{WildFlyImage, WildFlyImageRegistry};

pub fn console(matches: &ArgMatches, registry: &WildFlyImageRegistry) -> anyhow::Result<()> {
    let management_clients = get_management_clients(matches, registry)?;
    for client in management_clients {
        let url = format!("http://localhost:{}/console", client.management_port);
        webbrowser::open(&url)?;
    }
    Ok(())
}

fn get_management_clients(
    matches: &ArgMatches,
    registry: &WildFlyImageRegistry,
) -> anyhow::Result<Vec<ManagementClient>> {
    if let Some(name) = matches.get_one::<String>("name") {
        let wildfly_images = matches.get_one::<Vec<WildFlyImage>>("wildfly-version");
        if let Some(wildfly_images) = wildfly_images
            && wildfly_images.len() > 1
        {
            bail!("Option <name> is not allowed when multiple <wildfly-version> are specified!");
        }
        let instance = block_on(get_instance(
            wildfly_images.map(|v| v.as_slice()),
            Some(name),
            registry,
        ))?;
        Ok(vec![ManagementClient::from_container_instance(
            &instance, registry,
        )])
    } else if let Some(wildfly_images) = matches.get_one::<Vec<WildFlyImage>>("wildfly-version") {
        if wildfly_images.len() == 1 {
            Ok(vec![ManagementClient::custom_port(
                &wildfly_images[0],
                *matches
                    .get_one::<u16>("management")
                    .unwrap_or(&(wildfly_images[0].management_port())),
                registry,
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
            Ok(wildfly_images
                .iter()
                .map(|img| ManagementClient::default_port(img, registry))
                .collect())
        }
    } else {
        let containers = block_on(container_ps(
            vec![Standalone, DomainController],
            None,
            None,
            true,
            registry,
        ))?;
        Ok(containers
            .iter()
            .map(|c| ManagementClient::from_container_instance(c, registry))
            .collect())
    }
}
