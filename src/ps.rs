use crate::podman::podman_ps;
use crate::wildfly::ServerType::{DomainController, HostController, Standalone};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use futures::executor::block_on;

pub fn ps() -> anyhow::Result<()> {
    let mut instances = block_on(podman_ps(
        vec![Standalone, DomainController, HostController],
        None,
        None,
        true,
    ))?;
    instances.sort();
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "ID", "Version", "Type", "Name", "Ports", "Status", "Image",
        ]);
    for instance in instances {
        table.add_row(vec![
            Cell::new(instance.container_id),
            Cell::new(&instance.admin_container.wildfly_container.version).fg(Color::DarkMagenta),
            Cell::new(&instance.admin_container.server_type.short_name()).fg(Color::DarkCyan),
            Cell::new(instance.name).fg(Color::DarkYellow),
            if let Some(ports) = instance.ports {
                Cell::new(format!("{}/{}", ports.http, ports.management)).fg(Color::Green)
            } else {
                Cell::new("")
            },
            Cell::new(instance.status),
            Cell::new(instance.admin_container.image_name()).fg(Color::Grey),
        ]);
    }
    println!("{table}");
    Ok(())
}
