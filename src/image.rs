use crate::container::{container_images, container_ps};
use crate::wildfly::{AdminContainer, ServerType};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use console::style;
use futures::executor::block_on;
use std::collections::HashMap;
use std::process::Stdio;

pub fn images() -> anyhow::Result<()> {
    let mut images = AdminContainer::all_versions_by_image_name();
    block_on(local_images(&mut images))?;
    block_on(images_in_use(&mut images))?;
    let mut image_values = images.values().collect::<Vec<_>>();
    image_values.sort();

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Type", "Image"]);
    for image in image_values {
        let cells = vec![
            Cell::new(&image.wildfly_container.version).fg(Color::DarkMagenta),
            Cell::new(image.server_type.short_name()).fg(Color::DarkCyan),
            if image.in_use {
                Cell::new(image.image_name()).fg(Color::Green)
            } else if image.local_image {
                Cell::new(image.image_name())
            } else {
                Cell::new(image.image_name()).fg(Color::AnsiValue(248))
            },
        ];
        table.add_row(cells);
    }
    println!("{table}");
    println!(
        "Image name legend: {}, local, {}",
        style("in use").green(),
        style("remote").color256(248)
    );
    Ok(())
}

async fn local_images(images: &mut HashMap<String, AdminContainer>) -> anyhow::Result<()> {
    let mut command = container_images();
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    let output = String::from_utf8(output.stdout)?;
    for line in output.lines() {
        if let Some(image) = images.get_mut(line) {
            image.local_image = true
        }
    }
    Ok(())
}

async fn images_in_use(images: &mut HashMap<String, AdminContainer>) -> anyhow::Result<()> {
    let instances = container_ps(
        vec![
            ServerType::Standalone,
            ServerType::DomainController,
            ServerType::HostController,
        ],
        None,
        None,
        false,
    )
    .await?;
    for instance in instances {
        if let Some(image) = images.get_mut(&instance.admin_container.image_name()) {
            image.in_use = true
        }
    }
    Ok(())
}
