use crate::constants::{WILDFLY_ADMIN_CONTAINER, WILDFLY_ADMIN_CONTAINER_REPOSITORY};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

use super::ServerType;

/// A WildFly admin container combining a version with a server type and image metadata.
#[derive(Eq, PartialEq, Clone)]
pub struct AdminContainer {
    pub wildfly_container: WildFlyContainer,
    pub server_type: ServerType,
    pub local_image: bool,
    pub in_use: bool,
}

impl AdminContainer {
    pub fn new(wildfly_container: WildFlyContainer, server_type: ServerType) -> AdminContainer {
        AdminContainer {
            wildfly_container,
            server_type,
            local_image: false,
            in_use: false,
        }
    }

    pub fn domain(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::new(wildfly_container.clone(), ServerType::DomainController),
            AdminContainer::new(wildfly_container.clone(), ServerType::HostController),
        ]
    }

    pub fn all_types(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::new(wildfly_container.clone(), ServerType::Standalone),
            AdminContainer::new(wildfly_container.clone(), ServerType::DomainController),
            AdminContainer::new(wildfly_container.clone(), ServerType::HostController),
        ]
    }

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
