//! Lists all supported WildFly versions.

use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use wildfly_container_versions::VERSIONS;

pub fn versions() -> anyhow::Result<()> {
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

    for wc in VERSIONS.values() {
        table.add_row(vec![
            Cell::new(wc.display_version()).fg(Color::DarkMagenta),
            Cell::new(format!("{}{}", wc.version, suffix_display(&wc.suffix))),
            Cell::new(format!("{}{}", wc.core_version, suffix_display(&wc.suffix))),
            Cell::new(&wc.repository).fg(Color::AnsiValue(248)),
        ]);
    }

    println!("\n{table}");
    Ok(())
}

fn suffix_display(suffix: &str) -> String {
    if suffix.is_empty() {
        String::new()
    } else {
        format!(".{suffix}")
    }
}
