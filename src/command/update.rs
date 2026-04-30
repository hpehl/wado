use std::thread;

use wildfly_meta::{UpdateStatus, update_wildfly_images};

pub fn update() -> anyhow::Result<()> {
    let status = thread::spawn(update_wildfly_images)
        .join()
        .map_err(|_| anyhow::anyhow!("Update thread panicked"))??;
    match &status {
        UpdateStatus::Downloaded { version, count } => {
            println!(
                "WildFly images downloaded ({} entries, version {})",
                count, version
            );
        }
        UpdateStatus::Updated {
            from_version,
            to_version,
            diff,
        } => {
            println!(
                "WildFly images updated from version {} to {}",
                from_version, to_version
            );
            if !diff.added.is_empty() {
                println!("  Added: {}", diff.added.join(", "));
            }
            if !diff.removed.is_empty() {
                println!("  Removed: {}", diff.removed.join(", "));
            }
        }
        UpdateStatus::AlreadyUpToDate(version) => {
            println!("WildFly images already up to date (version {})", version);
        }
    }
    Ok(())
}
