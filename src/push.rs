use crate::args::admin_containers_argument;
use crate::command::{CommandStatus, summary};
use crate::podman::verify_podman;
use crate::progress::{Progress, stderr_reader};
use crate::wildfly::AdminContainer;
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub fn push(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_podman()?;

    let admin_containers = admin_containers_argument(matches);
    let chunk_size = *matches.get_one::<u16>("chunks").unwrap_or(&0);
    if chunk_size > 0 {
        push_chunks(&admin_containers, chunk_size)?;
    } else {
        push_all(&admin_containers)?;
    }
    Ok(())
}

fn push_chunks(admin_containers: &[AdminContainer], chunk_size: u16) -> anyhow::Result<()> {
    let count = admin_containers.len();
    let instant = Instant::now();
    let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count);
    let chunks = admin_containers.chunks(chunk_size as usize);
    for chunk in chunks {
        match block_on(start_push(chunk.to_vec())) {
            Ok(status) => {
                all_status.extend(status);
            }
            Err(_) => {
                // ignore the error and continue with the next chunk
                continue;
            }
        };
    }
    summary("Push", "images", count, instant, all_status);
    Ok(())
}

fn push_all(admin_containers: &[AdminContainer]) -> anyhow::Result<()> {
    let count = admin_containers.len();
    let instant = Instant::now();
    let status = block_on(start_push(admin_containers.to_vec()))?;
    summary("Push", "images", count, instant, status);
    Ok(())
}

async fn start_push(admin_containers: Vec<AdminContainer>) -> anyhow::Result<Vec<CommandStatus>> {
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

        let mut command = Command::new("podman");
        if !admin_container.wildfly_container.platforms.is_empty() {
            command.arg("manifest");
        }
        command.arg("push").arg(&admin_container.image_name());

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-push.");

        let stderr = stderr_reader(&mut child);
        let progress_clone = progress.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            let status = progress.finish(output, None);
            status
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stderr).await;
        });
    }

    // wait for all commands to finish
    Ok(commands.join_all().await)
}
