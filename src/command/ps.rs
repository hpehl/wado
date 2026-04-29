use crate::container::container_ps;
use crate::wildfly::ServerType::{DomainController, HostController, Standalone};
use clap::ArgMatches;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use futures::executor::block_on;
use wildfly_meta::WildFlyImageRegistry;

pub fn ps(matches: &ArgMatches, registry: &WildFlyImageRegistry) -> anyhow::Result<()> {
    let mut server_types = vec![];
    if matches.get_flag("standalone") {
        server_types.push(Standalone);
    }
    if matches.get_flag("domain") {
        server_types.push(DomainController);
        server_types.push(HostController);
    }
    if !matches.get_flag("standalone") && !matches.get_flag("domain") {
        server_types.push(Standalone);
        server_types.push(DomainController);
        server_types.push(HostController);
    }
    let mut instances = block_on(container_ps(server_types, None, None, true, registry))?;
    instances.sort();
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Version", "Type", "Name", "Config", "Ports", "Topology", "Status", "ID",
        ]);
    for instance in instances {
        table.add_row(vec![
            Cell::new(instance.admin_image.wildfly_image.short_name()).fg(Color::DarkMagenta),
            Cell::new(instance.admin_image.server_type.short_name()).fg(Color::DarkCyan),
            Cell::new(instance.name).fg(Color::DarkYellow),
            Cell::new(instance.config.as_deref().unwrap_or("")).fg(Color::DarkCyan),
            if let Some(ports) = instance.ports {
                Cell::new(format!("{}/{}", ports.http, ports.management)).fg(Color::Green)
            } else {
                Cell::new("")
            },
            Cell::new(instance.topology.as_deref().unwrap_or("")).fg(Color::DarkBlue),
            Cell::new(instance.status),
            Cell::new(instance.container_id).fg(Color::Grey),
        ]);
    }
    println!("{table}");
    Ok(())
}
