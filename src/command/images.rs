use crate::container::{container_images_cmd, container_ps};
use crate::wildfly::{AdminImage, ServerType};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use console::style;
use futures::executor::block_on;
use std::collections::HashSet;
use std::process::Stdio;
use wildfly_meta::WildFlyImageRegistry;

pub fn images(registry: &WildFlyImageRegistry) -> anyhow::Result<()> {
    let all = AdminImage::all_versions_by_image_name(registry);
    let local = block_on(local_image_names())?;
    let in_use = block_on(image_names_in_use(registry))?;
    let mut image_values: Vec<AdminImage> = all
        .into_values()
        .map(|ac| {
            let name = ac.image_name();
            AdminImage {
                local_image: local.contains(&name),
                in_use: in_use.contains(&name),
                ..ac
            }
        })
        .collect();
    image_values.sort();

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Type", "Image"]);
    for image in &image_values {
        let cells = vec![
            Cell::new(image.wildfly_image.short_name()).fg(Color::DarkMagenta),
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

async fn local_image_names() -> anyhow::Result<HashSet<String>> {
    let mut command = container_images_cmd();
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    let output = String::from_utf8(output.stdout)?;
    Ok(output.lines().map(String::from).collect())
}

async fn image_names_in_use(registry: &WildFlyImageRegistry) -> anyhow::Result<HashSet<String>> {
    let instances = container_ps(
        vec![
            ServerType::Standalone,
            ServerType::DomainController,
            ServerType::HostController,
        ],
        None,
        None,
        false,
        registry,
    )
    .await?;
    Ok(instances
        .iter()
        .map(|i| i.admin_image.image_name())
        .collect())
}
