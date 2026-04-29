mod common;
mod dev;
mod stable;

use crate::args::admin_images_argument;
use crate::constants::WILDFLY_ADMIN_CONTAINER;
use crate::container::verify_container_command;
use clap::ArgMatches;

pub async fn build(matches: &ArgMatches) -> anyhow::Result<()> {
    verify_container_command()?;
    let admin_images = admin_images_argument(matches);

    let has_dev = admin_images.iter().any(|ac| ac.wildfly_image.is_dev());
    let has_stable = admin_images.iter().any(|ac| !ac.wildfly_image.is_dev());
    if has_dev && has_stable {
        anyhow::bail!(
            "Cannot mix dev and versioned builds. \
             Use '{wado} build dev' or '{wado} build <versions>', but not both.",
            wado = WILDFLY_ADMIN_CONTAINER
        );
    }

    if has_dev {
        dev::build_dev(matches, admin_images).await
    } else {
        stable::build_stable(matches, admin_images)
    }
}
