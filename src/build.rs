use crate::args::{admin_containers_argument, username_password_argument};
use crate::constants::{
    ADD_USER, ALLOWED_ORIGINS, ENTRYPOINT, LABEL_NAME, NO_AUTH, WILDFLY_ADMIN_CONTAINER,
};
use crate::container::{container_command, container_command_name, verify_container_command};
use crate::progress::{CommandStatus, Progress, stdout_reader, summary};
use crate::resources::{
    DOMAIN_CONTROLLER_DOCKERFILE, DOMAIN_CONTROLLER_ENTRYPOINT_SH, HOST_CONTROLLER_DOCKERFILE,
    HOST_CONTROLLER_ENTRYPOINT_SH, STANDALONE_DOCKERFILE, STANDALONE_ENTRYPOINT_SH,
};
use crate::wildfly::{AdminContainer, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use tempfile::tempdir;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub fn build(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let temp_dir = tempdir()?;
    let (username, password) = username_password_argument(matches);

    let username_path = temp_dir.path().join("username");
    let mut username_file = File::create(username_path.clone())?;
    username_file.write_all(username.as_bytes())?;

    let password_path = temp_dir.path().join("password");
    let mut password_file = File::create(password_path.clone())?;
    password_file.write_all(password.as_bytes())?;

    let admin_containers = admin_containers_argument(matches);
    let chunk_size = *matches.get_one::<u16>("chunks").unwrap_or(&0);
    if chunk_size > 0 {
        build_chunks(
            &admin_containers,
            &username_path,
            &password_path,
            chunk_size,
        )?;
    } else {
        build_all(&admin_containers, &username_path, &password_path)?;
    }
    temp_dir.close()?;
    Ok(())
}

fn build_chunks(
    admin_containers: &[AdminContainer],
    username_path: &Path,
    password_path: &Path,
    chunk_size: u16,
) -> anyhow::Result<()> {
    let count = admin_containers.len();
    let instant = Instant::now();
    let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count);
    let chunks = admin_containers.chunks(chunk_size as usize);
    for chunk in chunks {
        match block_on(start_builds(chunk.to_vec(), username_path, password_path)) {
            Ok(status) => {
                all_status.extend(status);
            }
            Err(_) => {
                // ignore the error and continue with the next chunk
                continue;
            }
        };
    }
    summary("Build", "images", count, instant, all_status);
    Ok(())
}

fn build_all(
    admin_containers: &[AdminContainer],
    username_path: &Path,
    password_path: &Path,
) -> anyhow::Result<()> {
    let count = admin_containers.len();
    let instant = Instant::now();
    let status = block_on(start_builds(
        admin_containers.to_vec(),
        username_path,
        password_path,
    ))?;
    summary("Build", "images", count, instant, status);
    Ok(())
}

async fn start_builds(
    admin_containers: Vec<AdminContainer>,
    username_path: &Path,
    password_path: &Path,
) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for admin_container in admin_containers {
        // TODO dev build
        if admin_container.wildfly_container.is_dev() {
            continue;
        }

        let progress = Progress::join(
            &multi_progress,
            &admin_container.wildfly_container.short_version,
            &admin_container.image_name(),
        );

        let temp_dir = tempdir()?;
        let mut child = podman_build(
            &admin_container,
            temp_dir.as_ref(),
            username_path,
            password_path,
        )?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Unable to run podman-build.");

        let stdout = stdout_reader(&mut child);
        let progress_clone = progress.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            let status = progress.finish(output, None);
            temp_dir
                .close()
                .expect("Unable to close temporary directory.");
            status
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stdout).await;
        });
    }

    // wait for all commands to finish
    Ok(commands.join_all().await)
}

fn podman_build(
    admin_container: &AdminContainer,
    context_dir: &Path,
    username_path: &Path,
    password_path: &Path,
) -> anyhow::Result<Command> {
    let entrypoint_path = context_dir.join(format!("{}-entrypoint.sh", WILDFLY_ADMIN_CONTAINER));
    let mut entrypoint_file = File::create(entrypoint_path)?;
    let dockerfile = match admin_container.server_type {
        ServerType::Standalone => {
            entrypoint_file.write_all(STANDALONE_ENTRYPOINT_SH.as_bytes())?;
            STANDALONE_DOCKERFILE
        }
        ServerType::DomainController => {
            entrypoint_file.write_all(DOMAIN_CONTROLLER_ENTRYPOINT_SH.as_bytes())?;
            DOMAIN_CONTROLLER_DOCKERFILE
        }
        ServerType::HostController => {
            entrypoint_file.write_all(HOST_CONTROLLER_ENTRYPOINT_SH.as_bytes())?;
            HOST_CONTROLLER_DOCKERFILE
        }
    };

    let dockerfile_path = context_dir.join("Dockerfile");
    let dockerfile_file = File::create(dockerfile_path.clone())?;
    let mut data = HashMap::new();
    data.insert("base-image", admin_container.wildfly_container.image_name());
    data.insert("label-name", LABEL_NAME.to_string());
    data.insert("label-value", admin_container.identifier());
    data.insert("entrypoint", ENTRYPOINT.to_string());
    data.insert("add-user", ADD_USER.to_string());
    data.insert("allowed-origins", ALLOWED_ORIGINS.to_string());
    data.insert("no-auth", NO_AUTH.to_string());
    if admin_container.wildfly_container.version.major < 27 {
        data.insert("primary", "master".to_string());
        data.insert("secondary", "slave".to_string());
    } else {
        data.insert("primary", "primary".to_string());
        data.insert("secondary", "secondary".to_string());
    }
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_string("dockerfile", dockerfile)?;
    hbs.render_template_to_write(dockerfile, &data, dockerfile_file)?;

    let command = if admin_container.wildfly_container.platforms.is_empty() {
        let mut command = container_command()?;
        command
            .arg("build")
            .arg("--secret")
            .arg(format!("id=username,src={}", username_path.display()))
            .arg("--secret")
            .arg(format!("id=password,src={}", password_path.display()))
            .arg("--tag")
            .arg(admin_container.image_name())
            .arg(context_dir.as_os_str().to_str().unwrap());
        command
    } else {
        let shell = if env::consts::OS == "windows" {
            "cmd"
        } else {
            "sh"
        };
        let commands = if env::consts::OS == "windows" {
            "/C"
        } else {
            "-c"
        };
        let mut command = Command::new(shell);
        command.arg(commands).arg(format!(
            "{command_name} manifest create --amend {image} && \
             {command_name} build --platform {platforms} \
                 --secret id=username,src={username} \
                 --secret id=password,src={password} \
                 --manifest {image} {context}",
            command_name = container_command_name()?,
            image = admin_container.image_name(),
            platforms = admin_container.wildfly_container.platforms.join(","),
            username = username_path.display(),
            password = password_path.display(),
            context = context_dir.display()
        ));
        command
    };
    Ok(command)
}
