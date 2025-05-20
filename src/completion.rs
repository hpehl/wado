use wildfly_container_versions::VERSIONS;

pub fn version_completion() -> anyhow::Result<()> {
    VERSIONS.values().for_each(|wfc| {
        if wfc.version.minor == 0 {
            println!("{}", wfc.version.major)
        } else {
            println!("{}.{}", wfc.version.major, wfc.version.minor)
        }
    });
    Ok(())
}
