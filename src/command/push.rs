use crate::args::admin_images_argument;
use crate::container::{container_command, verify_container_command};
use crate::progress::{CommandStatus, Progress, stderr_reader, summary};
use crate::wildfly::AdminImage;
use clap::ArgMatches;
use futures::executor::block_on;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub fn push(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;

    let admin_images = admin_images_argument(matches);
    let chunk_size = *matches.get_one::<u16>("chunks").unwrap_or(&0);
    if chunk_size > 0 {
        push_chunks(&admin_images, chunk_size)?;
    } else {
        push_all(&admin_images)?;
    }
    Ok(())
}

fn push_chunks(admin_images: &[AdminImage], chunk_size: u16) -> anyhow::Result<()> {
    let count = admin_images.len();
    let instant = Instant::now();
    let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count);
    let chunks = admin_images.chunks(chunk_size as usize);
    for chunk in chunks {
        match block_on(start_push(chunk.to_vec())) {
            Ok(status) => {
                all_status.extend(status);
            }
            Err(e) => {
                eprintln!("Chunk push failed: {}", e);
                continue;
            }
        };
    }
    summary("Push", "images", count, instant, all_status);
    Ok(())
}

fn push_all(admin_images: &[AdminImage]) -> anyhow::Result<()> {
    let count = admin_images.len();
    let instant = Instant::now();
    let status = block_on(start_push(admin_images.to_vec()))?;
    summary("Push", "images", count, instant, status);
    Ok(())
}

async fn start_push(admin_images: Vec<AdminImage>) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut commands = JoinSet::new();

    for admin_image in admin_images {
        let progress = Progress::join(
            &multi_progress,
            &admin_image.wildfly_image.short_name(),
            &admin_image.image_name(),
        );

        let mut command = container_command()?;
        if admin_image.wildfly_image.is_dev()
            || !admin_image.wildfly_image.platforms.is_empty()
        {
            command.arg("manifest");
        }
        command.arg("push").arg(admin_image.image_name());

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Unable to run podman-push.");

        let stderr = stderr_reader(&mut child);
        let progress_clone = progress.clone();
        commands.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish(output, None)
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stderr).await;
        });
    }

    // wait for all commands to finish
    Ok(commands.join_all().await)
}
