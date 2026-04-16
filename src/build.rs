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

// ------------------------------------------------------ shared build helpers

pub fn write_entrypoint(context_dir: &Path, server_type: &ServerType) -> anyhow::Result<()> {
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

pub fn base_template_data(admin_container: &AdminContainer) -> HashMap<&'static str, String> {
    let mut data = HashMap::new();
    data.insert("label-name", LABEL_NAME.to_string());
    data.insert("label-value", admin_container.identifier());
    data.insert("entrypoint", ENTRYPOINT.to_string());
    data.insert("add-user", ADD_USER.to_string());
    data.insert("allowed-origins", ALLOWED_ORIGINS.to_string());
    data.insert("no-auth", NO_AUTH.to_string());
    data
}

pub fn render_dockerfile(
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

pub fn container_build_command(
    image_name: &str,
    platforms: &[String],
    username_path: &Path,
    password_path: &Path,
    context_dir: &Path,
) -> anyhow::Result<Command> {
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
        Ok(command)
    } else {
        let shell = if env::consts::OS == "windows" {
            "cmd"
        } else {
            "sh"
        };
        let shell_flag = if env::consts::OS == "windows" {
            "/C"
        } else {
            "-c"
        };
        let mut command = Command::new(shell);
        command.arg(shell_flag).arg(format!(
            "{command_name} manifest create --amend {image} && \
             {command_name} build --platform {platforms} \
                 --secret id=username,src={username} \
                 --secret id=password,src={password} \
                 --manifest {image} {context}",
            command_name = container_command_name()?,
            image = image_name,
            platforms = platforms.join(","),
            username = username_path.display(),
            password = password_path.display(),
            context = context_dir.display()
        ));
        Ok(command)
    }
}

pub async fn run_builds_verbose(
    admin_containers: &[AdminContainer],
    build_fn: impl Fn(&AdminContainer, &Path) -> anyhow::Result<Command>,
) -> anyhow::Result<Vec<CommandStatus>> {
    let mut statuses = Vec::new();

    for admin_container in admin_containers {
        let image_name = admin_container.image_name();
        println!("\n--- {} ---", image_name);

        let temp_dir = tempdir()?;
        let status = build_fn(admin_container, temp_dir.as_ref())?
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

// ------------------------------------------------------ build

pub async fn build(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;
    let admin_containers = admin_containers_argument(matches);

    let has_dev = admin_containers
        .iter()
        .any(|ac| ac.wildfly_container.is_dev());
    let has_stable = admin_containers
        .iter()
        .any(|ac| !ac.wildfly_container.is_dev());
    if has_dev && has_stable {
        anyhow::bail!(
            "Cannot mix dev and versioned builds. \
             Use '{wado} build dev' or '{wado} build <versions>', but not both.",
            wado = WILDFLY_ADMIN_CONTAINER
        );
    }

    if has_dev {
        crate::dev::build_dev(matches, admin_containers).await
    } else {
        build_stable(matches, admin_containers)
    }
}

fn build_stable(matches: &ArgMatches, admin_containers: Vec<AdminContainer>) -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let (username, password) = username_password_argument(matches);
    let verbose = matches.get_flag("verbose");

    let username_path = temp_dir.path().join("username");
    let mut username_file = File::create(username_path.clone())?;
    username_file.write_all(username.as_bytes())?;

    let password_path = temp_dir.path().join("password");
    let mut password_file = File::create(password_path.clone())?;
    password_file.write_all(password.as_bytes())?;

    let chunk_size = *matches.get_one::<u16>("chunks").unwrap_or(&0);
    let count = admin_containers.len();
    let instant = Instant::now();

    let status = if verbose {
        block_on(start_builds_verbose(
            admin_containers,
            &username_path,
            &password_path,
        ))?
    } else if chunk_size > 0 {
        let mut all_status = Vec::new();
        for chunk in admin_containers.chunks(chunk_size as usize) {
            match block_on(start_builds(chunk.to_vec(), &username_path, &password_path)) {
                Ok(status) => all_status.extend(status),
                Err(e) => {
                    eprintln!("Chunk build failed: {}", e);
                    continue;
                }
            }
        }
        all_status
    } else {
        block_on(start_builds(
            admin_containers,
            &username_path,
            &password_path,
        ))?
    };

    summary("Build", "images", count, instant, status);
    temp_dir.close()?;
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
        let progress = Progress::join(
            &multi_progress,
            &admin_container.version_label(),
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

async fn start_builds_verbose(
    admin_containers: Vec<AdminContainer>,
    username_path: &Path,
    password_path: &Path,
) -> anyhow::Result<Vec<CommandStatus>> {
    run_builds_verbose(&admin_containers, |ac, dir| {
        podman_build(ac, dir, username_path, password_path)
    })
    .await
}

fn podman_build(
    admin_container: &AdminContainer,
    context_dir: &Path,
    username_path: &Path,
    password_path: &Path,
) -> anyhow::Result<Command> {
    write_entrypoint(context_dir, &admin_container.server_type)?;

    let dockerfile = match admin_container.server_type {
        ServerType::Standalone => STANDALONE_DOCKERFILE,
        ServerType::DomainController => DOMAIN_CONTROLLER_DOCKERFILE,
        ServerType::HostController => HOST_CONTROLLER_DOCKERFILE,
    };

    let mut data = base_template_data(admin_container);
    data.insert("base-image", admin_container.wildfly_container.image_name());
    if !admin_container.wildfly_container.is_dev()
        && admin_container.wildfly_container.version.major < 27
    {
        data.insert("primary", "master".to_string());
        data.insert("secondary", "slave".to_string());
    } else {
        data.insert("primary", "primary".to_string());
        data.insert("secondary", "secondary".to_string());
    }

    render_dockerfile(context_dir, dockerfile, &data)?;
    container_build_command(
        &admin_container.image_name(),
        &admin_container.wildfly_container.platforms,
        username_path,
        password_path,
        context_dir,
    )
}
