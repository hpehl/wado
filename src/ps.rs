use crate::container::container_ps;
use crate::wildfly::ServerType::{DomainController, HostController, Standalone};
use clap::ArgMatches;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use futures::executor::block_on;

pub fn ps(matches: &ArgMatches) -> anyhow::Result<()> {
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
    let mut instances = block_on(container_ps(server_types, None, None, true))?;
    instances.sort();
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Type", "Name", "Ports", "Status", "ID"]);
    for instance in instances {
        table.add_row(vec![
            Cell::new(&instance.admin_container.wildfly_container.version).fg(Color::DarkMagenta),
            Cell::new(instance.admin_container.server_type.short_name()).fg(Color::DarkCyan),
            Cell::new(instance.name).fg(Color::DarkYellow),
            if let Some(ports) = instance.ports {
                Cell::new(format!("{}/{}", ports.http, ports.management)).fg(Color::Green)
            } else {
                Cell::new("")
            },
            Cell::new(instance.status),
            Cell::new(instance.container_id).fg(Color::Grey),
        ]);
    }
    println!("{table}");
    Ok(())
}
