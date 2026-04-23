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

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(version: &str) -> WildFlyContainer {
        WildFlyContainer::version(version).unwrap()
    }

    #[test]
    fn new_sets_default_flags() {
        let ac = AdminContainer::new(wc("39"), ServerType::Standalone);
        assert!(!ac.local_image);
        assert!(!ac.in_use);
        assert_eq!(ac.server_type, ServerType::Standalone);
    }

    #[test]
    fn domain_creates_dc_and_hc() {
        let containers = AdminContainer::domain(wc("39"));
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].server_type, ServerType::DomainController);
        assert_eq!(containers[1].server_type, ServerType::HostController);
    }

    #[test]
    fn all_types_creates_three() {
        let containers = AdminContainer::all_types(wc("39"));
        assert_eq!(containers.len(), 3);
        assert_eq!(containers[0].server_type, ServerType::Standalone);
        assert_eq!(containers[1].server_type, ServerType::DomainController);
        assert_eq!(containers[2].server_type, ServerType::HostController);
    }

    #[test]
    fn identifier_stable_version() {
        let ac = AdminContainer::new(wc("39"), ServerType::Standalone);
        assert_eq!(ac.identifier(), "sa-390");

        let ac = AdminContainer::new(wc("35"), ServerType::DomainController);
        assert_eq!(ac.identifier(), "dc-350");
    }

    #[test]
    fn identifier_dev_version() {
        let ac = AdminContainer::new(wc("dev"), ServerType::Standalone);
        assert_eq!(ac.identifier(), "sa-dev");
    }

    #[test]
    fn image_name_stable() {
        let ac = AdminContainer::new(wc("39"), ServerType::Standalone);
        let name = ac.image_name();
        assert!(name.starts_with("quay.io/wado/wado-sa:"));
        assert!(name.contains("39.0"));
    }

    #[test]
    fn image_name_dev() {
        let ac = AdminContainer::new(wc("dev"), ServerType::HostController);
        let name = ac.image_name();
        assert!(name.starts_with("quay.io/wado/wado-hc:"));
        assert!(name.contains(wildfly_container_versions::DEVELOPMENT_TAG));
    }

    #[test]
    fn container_name_format() {
        let ac = AdminContainer::new(wc("39"), ServerType::Standalone);
        assert_eq!(ac.container_name(), "wado-sa-390");

        let ac = AdminContainer::new(wc("dev"), ServerType::DomainController);
        assert_eq!(ac.container_name(), "wado-dc-dev");
    }

    #[test]
    fn from_identifier_valid_stable() {
        let ac = AdminContainer::from_identifier("sa-390".to_string());
        assert!(ac.is_some());
        let ac = ac.unwrap();
        assert_eq!(ac.server_type, ServerType::Standalone);
        assert_eq!(ac.wildfly_container.identifier, 390);
    }

    #[test]
    fn from_identifier_valid_dev() {
        let ac = AdminContainer::from_identifier("dc-dev".to_string());
        assert!(ac.is_some());
        let ac = ac.unwrap();
        assert_eq!(ac.server_type, ServerType::DomainController);
        assert!(ac.wildfly_container.is_dev());
    }

    #[test]
    fn from_identifier_invalid_no_dash() {
        assert!(AdminContainer::from_identifier("sa390".to_string()).is_none());
    }

    #[test]
    fn from_identifier_invalid_server_type() {
        assert!(AdminContainer::from_identifier("xx-390".to_string()).is_none());
    }

    #[test]
    fn from_identifier_invalid_version() {
        assert!(AdminContainer::from_identifier("sa-999".to_string()).is_none());
    }

    #[test]
    fn ordering_dev_after_stable() {
        let stable = AdminContainer::new(wc("39"), ServerType::Standalone);
        let dev = AdminContainer::new(wc("dev"), ServerType::Standalone);
        assert!(stable < dev);
    }

    #[test]
    fn ordering_same_version_by_server_type() {
        let sa = AdminContainer::new(wc("39"), ServerType::Standalone);
        let dc = AdminContainer::new(wc("39"), ServerType::DomainController);
        let hc = AdminContainer::new(wc("39"), ServerType::HostController);
        assert!(sa < dc);
        assert!(dc < hc);
    }

    #[test]
    fn ordering_by_version() {
        let v35 = AdminContainer::new(wc("35"), ServerType::Standalone);
        let v39 = AdminContainer::new(wc("39"), ServerType::Standalone);
        assert!(v35 < v39);
    }

    #[test]
    fn all_versions_by_image_name_not_empty() {
        let map = AdminContainer::all_versions_by_image_name();
        assert!(!map.is_empty());
        for (key, ac) in &map {
            assert_eq!(key, &ac.image_name());
        }
    }
}
