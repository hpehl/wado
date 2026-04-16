mod source;
mod task;

use super::common::{
    base_template_data, container_build_commands, render_dockerfile, run_builds_verbose,
    run_preconditions, write_entrypoint,
};
use crate::args::username_password_argument;
use crate::container::container_command;
use crate::progress::CommandStatus;
use crate::resources::{
    DEV_DOMAIN_CONTROLLER_DOCKERFILE, DEV_HOST_CONTROLLER_DOCKERFILE, DEV_STANDALONE_DOCKERFILE,
};
use crate::wildfly::{AdminContainer, ServerType};
use clap::ArgMatches;
use console::{Emoji, style};
use indicatif::{HumanDuration, MultiProgress};
use source::{
    clone_and_build_repos, clone_and_build_repos_verbose, extract_hal_jar, extract_wildfly_dist,
    integrate_hal,
};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use task::DevTask;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::Instant;
use wildfly_container_versions::VERSIONS;

// ------------------------------------------------------ emojis & constants

static HAMMER: Emoji<'_, '_> = Emoji("\u{1f6e0}\u{fe0f}  ", "");
static LINK: Emoji<'_, '_> = Emoji("\u{1f517}  ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("\u{1f4e6}  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("\u{2728}  ", ":-)  ");

fn latest_platforms() -> Vec<String> {
    VERSIONS
        .last_key_value()
        .map(|(_, wfc)| wfc.platforms.clone())
        .unwrap_or_default()
}

// ------------------------------------------------------ config

struct DevBuildConfig<'a> {
    username_path: &'a Path,
    password_path: &'a Path,
    wildfly_branch: &'a str,
    hal_branch: &'a str,
    verbose: bool,
}

// ------------------------------------------------------ build dev

pub(in crate::build) async fn build_dev(
    matches: &ArgMatches,
    admin_containers: Vec<AdminContainer>,
) -> anyhow::Result<()> {
    let wildfly_branch = matches
        .get_one::<String>("wildfly-branch")
        .map(|s| s.as_str())
        .unwrap_or("main");
    let hal_branch = matches
        .get_one::<String>("hal-branch")
        .map(|s| s.as_str())
        .unwrap_or("main");
    let verbose = matches.get_flag("verbose");

    let temp_dir = tempdir()?;
    let (username, password) = username_password_argument(matches);

    let username_path = temp_dir.path().join("username");
    let mut username_file = File::create(username_path.clone())?;
    username_file.write_all(username.as_bytes())?;

    let password_path = temp_dir.path().join("password");
    let mut password_file = File::create(password_path.clone())?;
    password_file.write_all(password.as_bytes())?;

    let config = DevBuildConfig {
        username_path: &username_path,
        password_path: &password_path,
        wildfly_branch,
        hal_branch,
        verbose,
    };

    let instant = Instant::now();
    let statuses = run_dev_build(&config, admin_containers).await?;

    let failed: Vec<_> = statuses.iter().filter(|s| !s.success).collect();
    if failed.is_empty() {
        println!(
            "\n{}Done in {}",
            SPARKLE,
            style(HumanDuration(instant.elapsed())).cyan()
        );
    } else {
        println!();
        for s in &failed {
            println!(
                "  {} {}: {}",
                style("\u{2717}").red().bold(),
                style(&s.identifier).cyan(),
                style(&s.error_message).red()
            );
        }
        println!("\n{} container(s) failed", style(failed.len()).red().bold());
    }

    temp_dir.close()?;
    Ok(())
}

async fn run_dev_build(
    config: &DevBuildConfig<'_>,
    admin_containers: Vec<AdminContainer>,
) -> anyhow::Result<Vec<CommandStatus>> {
    let pid = std::process::id();
    let wf_volume = format!("wado-wildfly-build-{}", pid);
    let hal_volume = format!("wado-hal-build-{}", pid);

    // Create named volumes (avoids virtiofs bind-mount issues on macOS)
    create_volume(&wf_volume).await?;
    create_volume(&hal_volume).await?;

    let result = run_dev_build_inner(config, admin_containers, &wf_volume, &hal_volume).await;

    // Always clean up volumes
    remove_volume(&wf_volume).await;
    remove_volume(&hal_volume).await;

    result
}

async fn run_dev_build_inner(
    config: &DevBuildConfig<'_>,
    admin_containers: Vec<AdminContainer>,
    wf_volume: &str,
    hal_volume: &str,
) -> anyhow::Result<Vec<CommandStatus>> {
    // Phase 1: Clone and build both repos in parallel (inside containers, using named volumes)
    println!(
        "{} {}Cloning and building from source...",
        style("[1/3]").bold().dim(),
        HAMMER
    );
    if config.verbose {
        clone_and_build_repos_verbose(
            config.wildfly_branch,
            config.hal_branch,
            wf_volume,
            hal_volume,
        )
        .await?;
    } else {
        clone_and_build_repos(
            config.wildfly_branch,
            config.hal_branch,
            wf_volume,
            hal_volume,
        )
        .await?;
    }

    // Phase 2: Extract artifacts, integrate HAL
    println!(
        "{} {}Integrating HAL console...",
        style("[2/3]").bold().dim(),
        LINK
    );
    let artifact_dir = tempdir()?;
    let wildfly_dist = extract_wildfly_dist(wf_volume, artifact_dir.path()).await?;
    let hal_jar = extract_hal_jar(hal_volume, artifact_dir.path()).await?;
    integrate_hal(&wildfly_dist, &hal_jar)?;
    println!(
        "  {} {}",
        style("\u{2713}").green().bold(),
        style("HAL console integrated").cyan()
    );

    // Phase 3: Build container images
    println!(
        "{} {}Building containers...",
        style("[3/3]").bold().dim(),
        PACKAGE
    );
    if config.verbose {
        build_containers_verbose(
            admin_containers,
            config.username_path,
            config.password_path,
            &wildfly_dist,
        )
        .await
    } else {
        build_containers(
            admin_containers,
            config.username_path,
            config.password_path,
            &wildfly_dist,
        )
        .await
    }
}

// ------------------------------------------------------ volume management

async fn create_volume(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("volume").arg("create").arg(name);
    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create volume {}", name);
    }
    Ok(())
}

async fn remove_volume(name: &str) {
    if let Ok(mut cmd) = container_command() {
        cmd.arg("volume")
            .arg("rm")
            .arg("-f")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .ok();
    }
}

// ------------------------------------------------------ container build (progress)

async fn build_containers(
    admin_containers: Vec<AdminContainer>,
    username_path: &Path,
    password_path: &Path,
    wildfly_dist: &Path,
) -> anyhow::Result<Vec<CommandStatus>> {
    let multi = MultiProgress::new();
    let mut statuses = Vec::new();

    for admin_container in &admin_containers {
        let mut task = DevTask::new(&multi, &admin_container.image_name());
        task.set_progress("building container...");

        let temp_dir = tempdir()?;
        let mut child = run_preconditions(dev_podman_build(
            admin_container,
            temp_dir.as_ref(),
            username_path,
            password_path,
            wildfly_dist,
        )?)
        .await?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Unable to run container build: {}", e))?;

        let stdout = child.stdout.take().expect("No stdout handle");
        let mut stdout_lines = BufReader::new(stdout).lines();
        while let Some(line) = stdout_lines.next_line().await? {
            task.append_line(&line);
            let trimmed = line.trim();
            if let Some(step) = trimmed.strip_prefix("STEP ")
                && let Some(colon_pos) = step.find(':')
            {
                task.set_progress(&format!("STEP {}", &step[..colon_pos]));
            }
        }

        let output = child.wait_with_output().await;
        match output {
            Ok(output) => {
                if output.status.success() {
                    task.finish_success(None);
                    statuses.push(CommandStatus::success(&admin_container.image_name()));
                } else {
                    let err = format!("exit code {}", output.status.code().unwrap_or(-1));
                    task.finish_error(&err);
                    task.print_errors();
                    statuses.push(CommandStatus::error(
                        &admin_container.image_name(),
                        String::from_utf8_lossy(&output.stderr)
                            .replace('\n', " ")
                            .as_str(),
                    ));
                }
            }
            Err(e) => {
                let err = format!("failed: {}", e);
                task.finish_error(&err);
                task.print_errors();
                statuses.push(CommandStatus::error(&admin_container.image_name(), &err));
            }
        }

        temp_dir.close()?;
    }

    Ok(statuses)
}

// ------------------------------------------------------ container build (verbose)

async fn build_containers_verbose(
    admin_containers: Vec<AdminContainer>,
    username_path: &Path,
    password_path: &Path,
    wildfly_dist: &Path,
) -> anyhow::Result<Vec<CommandStatus>> {
    run_builds_verbose(&admin_containers, |ac, dir| {
        dev_podman_build(ac, dir, username_path, password_path, wildfly_dist)
    })
    .await
}

// ------------------------------------------------------ podman build command

fn dev_podman_build(
    admin_container: &AdminContainer,
    context_dir: &Path,
    username_path: &Path,
    password_path: &Path,
    wildfly_dist: &Path,
) -> anyhow::Result<Vec<tokio::process::Command>> {
    // Copy WildFly distribution into context directory
    let context_wildfly = context_dir.join("wildfly");
    copy_dir_recursive(wildfly_dist, &context_wildfly)?;

    write_entrypoint(context_dir, &admin_container.server_type)?;

    let dockerfile = match admin_container.server_type {
        ServerType::Standalone => DEV_STANDALONE_DOCKERFILE,
        ServerType::DomainController => DEV_DOMAIN_CONTROLLER_DOCKERFILE,
        ServerType::HostController => DEV_HOST_CONTROLLER_DOCKERFILE,
    };

    let data = base_template_data(admin_container);
    render_dockerfile(context_dir, dockerfile, &data)?;
    container_build_commands(
        &admin_container.image_name(),
        &latest_platforms(),
        username_path,
        password_path,
        context_dir,
    )
}

// ------------------------------------------------------ utility

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if entry_type.is_symlink() {
            let target = fs::read_link(entry.path())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dest_path)?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&target, &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
