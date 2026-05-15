//! Lists all supported WildFly versions.

use crate::json::VersionInfo;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use wildfly_meta::WildFlyImageRegistry;

pub fn versions(registry: &WildFlyImageRegistry, json: bool) -> anyhow::Result<()> {
    if json {
        let infos: Vec<VersionInfo> = registry
            .all()
            .iter()
            .map(|wc| VersionInfo {
                version: wc.short_name(),
                wildfly_version: wc.release_version.clone(),
                core_version: wc.core_release_version.clone(),
                repository: wc.repository.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string(&infos)?);
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Version",
            "WildFly Version",
            "WildFly Core",
            "Repository",
        ]);

    for wc in registry.all() {
        table.add_row(vec![
            Cell::new(wc.short_name()).fg(Color::DarkMagenta),
            Cell::new(&wc.release_version),
            Cell::new(&wc.core_release_version),
            Cell::new(&wc.repository).fg(Color::AnsiValue(248)),
        ]);
    }

    println!("\n{table}");
    Ok(())
}
