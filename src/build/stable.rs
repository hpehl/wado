use super::common::{
    base_template_data, container_build_commands, render_dockerfile, run_builds_verbose,
    run_preconditions, write_entrypoint,
};
use crate::args::username_password_argument;
use crate::progress::{CommandStatus, Progress, stdout_reader, summary};
use crate::resources::{
    DOMAIN_CONTROLLER_DOCKERFILE, HOST_CONTROLLER_DOCKERFILE, STANDALONE_DOCKERFILE,
};
use crate::wildfly::{AdminContainer, ServerType};
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use tempfile::tempdir;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub(super) fn build_stable(
    matches: &ArgMatches,
    admin_containers: Vec<AdminContainer>,
) -> anyhow::Result<()> {
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
        let mut child = run_preconditions(podman_build(
            &admin_container,
            temp_dir.as_ref(),
            username_path,
            password_path,
        )?)
        .await?
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
) -> anyhow::Result<Vec<Command>> {
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
    container_build_commands(
        &admin_container.image_name(),
        &admin_container.wildfly_container.platforms,
        username_path,
        password_path,
        context_dir,
    )
}
