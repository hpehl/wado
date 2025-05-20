use wildfly_container_versions::VERSIONS;

pub fn version_completion() -> anyhow::Result<()> {
    VERSIONS
        .values()
        .for_each(|v| println!("{}", v.short_version));
    Ok(())
}
