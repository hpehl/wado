//! Admin container metadata combining WildFly version with server type.

use crate::constants::{WILDFLY_ADMIN_CONTAINER, WILDFLY_ADMIN_CONTAINER_REPOSITORY};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

use super::ServerType;

/// A WildFly admin container combining a version with a server type and image metadata.
///
/// This is the central type linking a [`WildFlyContainer`] version to a [`ServerType`],
/// and provides methods for generating image names, container names, and identifiers.
#[derive(Eq, PartialEq, Clone)]
pub struct AdminContainer {
    /// The WildFly version and base image metadata.
    pub wildfly_container: WildFlyContainer,
    /// The server operating mode (standalone, domain controller, or host controller).
    pub server_type: ServerType,
    /// Whether a local image exists for this container.
    pub local_image: bool,
    /// Whether a running container is using this image.
    pub in_use: bool,
}

impl AdminContainer {
    /// Creates a new admin container with default flags (`local_image` and `in_use` are `false`).
    pub fn new(wildfly_container: WildFlyContainer, server_type: ServerType) -> AdminContainer {
        AdminContainer {
            wildfly_container,
            server_type,
            local_image: false,
            in_use: false,
        }
    }

    /// Creates admin containers for both domain controller and host controller.
    pub fn domain(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::new(wildfly_container.clone(), ServerType::DomainController),
            AdminContainer::new(wildfly_container.clone(), ServerType::HostController),
        ]
    }

    /// Creates admin containers for all three server types.
    pub fn all_types(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::new(wildfly_container.clone(), ServerType::Standalone),
            AdminContainer::new(wildfly_container.clone(), ServerType::DomainController),
            AdminContainer::new(wildfly_container.clone(), ServerType::HostController),
        ]
    }

    /// Returns a map of all known admin containers indexed by their full image name.
    pub fn all_versions_by_image_name() -> HashMap<String, AdminContainer> {
        let mut result = HashMap::new();
        VERSIONS.values().for_each(|v| {
            AdminContainer::all_types(v.clone()).iter().for_each(|ac| {
                result.insert(ac.image_name(), ac.clone());
            });
        });
        if let Ok(dev) = WildFlyContainer::version("dev") {
            AdminContainer::all_types(dev).iter().for_each(|ac| {
                result.insert(ac.image_name(), ac.clone());
            });
        }
        result
    }

    /// Parses an identifier string (e.g. `"sa-390"` or `"dc-dev"`) into an admin container.
    pub fn from_identifier(identifier: String) -> Option<AdminContainer> {
        if identifier.contains('-') {
            let parts = identifier.split('-').collect::<Vec<&str>>();
            if parts.len() == 2
                && let Ok(server_type) = ServerType::from_str(parts[0])
            {
                if parts[1] == "dev" {
                    if let Ok(wildfly_container) = WildFlyContainer::version("dev") {
                        return Some(AdminContainer {
                            wildfly_container,
                            server_type,
                            local_image: false,
                            in_use: false,
                        });
                    }
                } else if let Ok(id) = parts[1].parse::<u16>()
                    && let Ok(wildfly_container) = WildFlyContainer::lookup(id)
                {
                    return Some(AdminContainer {
                        wildfly_container,
                        server_type,
                        local_image: false,
                        in_use: false,
                    });
                }
            }
        }
        None
    }

    /// Returns the short identifier (e.g. `"sa-390"` or `"dc-dev"`).
    pub fn identifier(&self) -> String {
        if self.wildfly_container.is_dev() {
            format!("{}-dev", self.server_type.short_name())
        } else {
            format!(
                "{}-{}",
                self.server_type.short_name(),
                self.wildfly_container.identifier
            )
        }
    }

    /// Returns the fully qualified image name (e.g. `"quay.io/wado/wado-sa:39.0.0.Final"`).
    pub fn image_name(&self) -> String {
        let base_name = format!(
            "{}/{}-{}",
            WILDFLY_ADMIN_CONTAINER_REPOSITORY,
            WILDFLY_ADMIN_CONTAINER,
            self.server_type.short_name()
        );
        if self.wildfly_container.is_dev() {
            format!(
                "{}:{}",
                base_name,
                wildfly_container_versions::DEVELOPMENT_TAG
            )
        } else {
            format!(
                "{}:{}.{}",
                base_name, self.wildfly_container.version, self.wildfly_container.suffix
            )
        }
    }

    /// Returns the default container name (e.g. `"wado-sa-390"`).
    pub fn container_name(&self) -> String {
        format!("{}-{}", WILDFLY_ADMIN_CONTAINER, self.identifier())
    }
}

impl Ord for AdminContainer {
    fn cmp(&self, other: &Self) -> Ordering {
        let a_dev = self.wildfly_container.is_dev();
        let b_dev = other.wildfly_container.is_dev();
        match (a_dev, b_dev) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => {
                let version_ord = self.wildfly_container.cmp(&other.wildfly_container);
                if version_ord == Ordering::Equal {
                    self.server_type.cmp(&other.server_type)
                } else {
                    version_ord
                }
            }
        }
    }
}

impl PartialOrd for AdminContainer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
