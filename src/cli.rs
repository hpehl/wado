use crate::args::username_password_argument;
use crate::constants::WILDFLY_ADMIN_CONTAINER;
use crate::container::{container_ps, get_instance};
use crate::progress::Progress;
use crate::wildfly::ManagementClient;
use crate::wildfly::ServerType::{DomainController, Standalone};
use anyhow::{anyhow, bail, Context};
use clap::ArgMatches;
use fs::{create_dir_all, File};
use futures::executor::block_on;
use std::env::temp_dir;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use which::which;
use wildfly_container_versions::WildFlyContainer;

pub fn cli(matches: &ArgMatches) -> anyhow::Result<()> {
    which("java").with_context(|| "java not found")?;

    let management_client = if let Some(name) = matches.get_one::<String>("name") {
        let mut v = vec![];
        let wildfly_containers = if let Some(wildfly_container) =
            matches.get_one::<WildFlyContainer>("wildfly-version")
        {
            v.push(wildfly_container.clone());
            Some(&v)
        } else {
            None
        };
        let instance = block_on(get_instance(wildfly_containers, Some(name)))?;
        ManagementClient::from_container_instance(&instance)
    } else if let Some(wildfly_container) = matches.get_one::<WildFlyContainer>("wildfly-version") {
        ManagementClient::custom_port(
            wildfly_container,
            *matches
                .get_one::<u16>("management")
                .unwrap_or(&wildfly_container.management_port()),
        )
    } else {
        let containers = block_on(container_ps(
            vec![Standalone, DomainController],
            None,
            None,
            true,
        ))?;
        if containers.is_empty() {
            bail!("No running containers found.")
        } else if containers.len() > 1 {
            bail!("Multiple running containers found. Please specify a version or a name.")
        } else {
            ManagementClient::from_container_instance(&containers[0])
        }
    };
    let (username, password) = username_password_argument(matches);
    let parameters = matches
        .get_many::<String>("cli-parameters")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>();
    let temp_dir = temp_dir().join(format!(
        "{}-cli-{}",
        WILDFLY_ADMIN_CONTAINER, management_client.wildfly_container.identifier
    ));

    create_dir_all(&temp_dir)?;
    block_on(connect_to_cli(
        &management_client,
        &temp_dir,
        username,
        password,
        parameters,
    ))
}

async fn connect_to_cli(
    management_client: &ManagementClient,
    cli_dir: &Path,
    username: &str,
    password: &str,
    parameters: Vec<String>,
) -> anyhow::Result<()> {
    let progress = Progress::new(
        &management_client.wildfly_container.short_version,
        &management_client.wildfly_container.image_name(),
    );

    let cli_jar = cli_dir.join("cli.jar");
    let cli_config = cli_dir.join("cli.xml");
    progress.show_progress("Downloading CLI jar and config...");
    let (jar_result, config_result) = futures::join!(
        download_file(management_client.cli_jar_url.as_str(), &cli_jar),
        download_file(management_client.cli_config_url.as_str(), &cli_config)
    );
    jar_result?;
    config_result?;

    progress.finish_no_output(None);
    let output = Command::new("java")
        .arg(format!(
            "-Djboss.cli.config={}",
            cli_config.as_os_str().to_str().unwrap()
        ))
        .arg("-jar")
        .arg(cli_jar)
        .arg(format!("--user={}", username))
        .arg(format!("--password={}", password))
        .arg(format!(
            "--controller=localhost:{}",
            management_client.management_port
        ))
        .arg("--connect")
        .args(parameters.clone())
        .spawn()?
        .wait_with_output()
        .await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "CLI failed with exit code: {}",
            output.status.code().unwrap()
        ))
    }
}

async fn download_file(url: &str, path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        Ok(())
    } else {
        let response = reqwest::get(url).await?;
        if response.status().is_success() {
            let mut file = File::create(path)?;
            let content = response.bytes().await?;
            file.write_all(&content)?;
            Ok(())
        } else {
            Err(anyhow!("Failed to download: {}", response.status()))
        }
    }
}
