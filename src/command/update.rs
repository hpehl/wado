use std::thread;

use wildfly_meta::update_wildfly_images;

pub fn update() -> anyhow::Result<()> {
    let status = thread::spawn(update_wildfly_images)
        .join()
        .map_err(|_| anyhow::anyhow!("Update thread panicked"))??;
    println!("{}", status.summary("WildFly images"));
    Ok(())
}
