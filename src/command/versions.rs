//! Lists all supported WildFly versions.

use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use wildfly_meta::WildFlyImageRegistry;

pub fn versions(registry: &WildFlyImageRegistry) -> anyhow::Result<()> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Version",
            "Full Version",
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
