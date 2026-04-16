use crate::constants::{
    ADD_USER, ALLOWED_ORIGINS, ENTRYPOINT, LABEL_NAME, NO_AUTH, WILDFLY_ADMIN_CONTAINER,
};
use crate::container::container_command;
use crate::progress::CommandStatus;
use crate::resources::{
    DOMAIN_CONTROLLER_ENTRYPOINT_SH, HOST_CONTROLLER_ENTRYPOINT_SH, STANDALONE_ENTRYPOINT_SH,
};
use crate::wildfly::{AdminContainer, ServerType};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use tempfile::tempdir;
use tokio::process::Command;

pub(super) fn write_entrypoint(
    context_dir: &Path,
    server_type: &ServerType,
) -> anyhow::Result<()> {
    let entrypoint_path = context_dir.join(format!("{}-entrypoint.sh", WILDFLY_ADMIN_CONTAINER));
    let mut entrypoint_file = File::create(entrypoint_path)?;
    let entrypoint_content = match server_type {
        ServerType::Standalone => STANDALONE_ENTRYPOINT_SH,
        ServerType::DomainController => DOMAIN_CONTROLLER_ENTRYPOINT_SH,
        ServerType::HostController => HOST_CONTROLLER_ENTRYPOINT_SH,
    };
    entrypoint_file.write_all(entrypoint_content.as_bytes())?;
    Ok(())
}

pub(super) fn base_template_data(
    admin_container: &AdminContainer,
) -> HashMap<&'static str, String> {
    let mut data = HashMap::new();
    data.insert("label-name", LABEL_NAME.to_string());
    data.insert("label-value", admin_container.identifier());
    data.insert("entrypoint", ENTRYPOINT.to_string());
    data.insert("add-user", ADD_USER.to_string());
    data.insert("allowed-origins", ALLOWED_ORIGINS.to_string());
    data.insert("no-auth", NO_AUTH.to_string());
    data
}

pub(super) fn render_dockerfile(
    context_dir: &Path,
    template: &str,
    data: &HashMap<&'static str, String>,
) -> anyhow::Result<()> {
    let dockerfile_path = context_dir.join("Dockerfile");
    let dockerfile_file = File::create(dockerfile_path)?;
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_string("dockerfile", template)?;
    hbs.render_template_to_write(template, data, dockerfile_file)?;
    Ok(())
}

pub(super) fn container_build_commands(
    image_name: &str,
    platforms: &[String],
    username_path: &Path,
    password_path: &Path,
    context_dir: &Path,
) -> anyhow::Result<Vec<Command>> {
    if platforms.is_empty() {
        let mut command = container_command()?;
        command
            .arg("build")
            .arg("--secret")
            .arg(format!("id=username,src={}", username_path.display()))
            .arg("--secret")
            .arg(format!("id=password,src={}", password_path.display()))
            .arg("--tag")
            .arg(image_name)
            .arg(context_dir.as_os_str().to_str().unwrap());
        Ok(vec![command])
    } else {
        let mut manifest_cmd = container_command()?;
        manifest_cmd
            .arg("manifest")
            .arg("create")
            .arg("--amend")
            .arg(image_name);

        let mut build_cmd = container_command()?;
        build_cmd
            .arg("build")
            .arg("--platform")
            .arg(platforms.join(","))
            .arg("--secret")
            .arg(format!("id=username,src={}", username_path.display()))
            .arg("--secret")
            .arg(format!("id=password,src={}", password_path.display()))
            .arg("--manifest")
            .arg(image_name)
            .arg(context_dir.as_os_str().to_str().unwrap());

        Ok(vec![manifest_cmd, build_cmd])
    }
}

pub(super) async fn run_preconditions(mut commands: Vec<Command>) -> anyhow::Result<Command> {
    let last = commands
        .pop()
        .ok_or_else(|| anyhow::anyhow!("no build commands"))?;
    for mut cmd in commands {
        let status = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Build preparation failed with {}", status);
        }
    }
    Ok(last)
}

pub(super) async fn run_builds_verbose(
    admin_containers: &[AdminContainer],
    build_fn: impl Fn(&AdminContainer, &Path) -> anyhow::Result<Vec<Command>>,
) -> anyhow::Result<Vec<CommandStatus>> {
    let mut statuses = Vec::new();

    for admin_container in admin_containers {
        let image_name = admin_container.image_name();
        println!("\n--- {} ---", image_name);

        let temp_dir = tempdir()?;
        let status = run_preconditions(build_fn(admin_container, temp_dir.as_ref())?)
            .await?
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await;

        match status {
            Ok(status) => {
                if status.success() {
                    println!("--- {} done ---\n", image_name);
                    statuses.push(CommandStatus::success(&image_name));
                } else {
                    let err = format!("exit code {}", status.code().unwrap_or(-1));
                    println!("--- {} FAILED: {} ---\n", image_name, err);
                    statuses.push(CommandStatus::error(&image_name, &err));
                }
            }
            Err(e) => {
                let err = format!("failed: {}", e);
                println!("--- {} FAILED: {} ---\n", image_name, err);
                statuses.push(CommandStatus::error(&image_name, &err));
            }
        }

        temp_dir.close()?;
    }

    Ok(statuses)
}
