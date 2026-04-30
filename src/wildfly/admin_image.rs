//! Admin image metadata combining WildFly version with server type.

use crate::constants::{WILDFLY_ADMIN_CONTAINER, WILDFLY_ADMIN_CONTAINER_REPOSITORY};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use wildfly_meta::{DEVELOPMENT_TAG, WildFlyImage, WildFlyImageRegistry, wildfly_dev};

use super::ServerType;

/// A WildFly admin image combining a version with a server type and image metadata.
///
/// This is the central type linking a [`WildFlyImage`] version to a [`ServerType`],
/// and provides methods for generating image names, container names, and identifiers.
#[derive(Eq, PartialEq, Clone)]
pub struct AdminImage {
    /// The WildFly version and base image metadata.
    pub wildfly_image: WildFlyImage,
    /// The server operating mode (standalone, domain controller, or host controller).
    pub server_type: ServerType,
    /// Whether a local image exists for this container.
    pub local_image: bool,
    /// Whether a running container is using this image.
    pub in_use: bool,
}

impl AdminImage {
    /// Creates a new admin image with default flags (`local_image` and `in_use` are `false`).
    pub fn new(wildfly_image: WildFlyImage, server_type: ServerType) -> AdminImage {
        AdminImage {
            wildfly_image,
            server_type,
            local_image: false,
            in_use: false,
        }
    }

    /// Creates admin images for both domain controller and host controller.
    pub fn domain(wildfly_image: WildFlyImage) -> Vec<AdminImage> {
        vec![
            AdminImage::new(wildfly_image.clone(), ServerType::DomainController),
            AdminImage::new(wildfly_image.clone(), ServerType::HostController),
        ]
    }

    /// Creates admin images for all three server types.
    pub fn all_types(wildfly_image: WildFlyImage) -> Vec<AdminImage> {
        vec![
            AdminImage::new(wildfly_image.clone(), ServerType::Standalone),
            AdminImage::new(wildfly_image.clone(), ServerType::DomainController),
            AdminImage::new(wildfly_image.clone(), ServerType::HostController),
        ]
    }

    /// Returns a map of all known admin images indexed by their full image name.
    pub fn all_versions_by_image_name(
        registry: &WildFlyImageRegistry,
    ) -> HashMap<String, AdminImage> {
        let mut result = HashMap::new();
        for img in registry.all() {
            for ai in AdminImage::all_types(img.clone()) {
                result.insert(ai.image_name(), ai);
            }
        }
        let dev = wildfly_dev();
        for ai in AdminImage::all_types(dev) {
            result.insert(ai.image_name(), ai);
        }
        result
    }

    /// Parses an identifier string (e.g. `"sa-390"` or `"dc-dev"`) into an admin image.
    pub fn from_identifier(
        identifier: String,
        registry: &WildFlyImageRegistry,
    ) -> Option<AdminImage> {
        if identifier.contains('-') {
            let parts = identifier.split('-').collect::<Vec<&str>>();
            if parts.len() == 2
                && let Ok(server_type) = ServerType::from_str(parts[0])
            {
                if parts[1] == "dev" {
                    return Some(AdminImage {
                        wildfly_image: wildfly_dev(),
                        server_type,
                        local_image: false,
                        in_use: false,
                    });
                } else if let Ok(id) = parts[1].parse::<u16>()
                    && let Some(wildfly_image) = registry.get(id).cloned()
                {
                    return Some(AdminImage {
                        wildfly_image,
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
        if self.wildfly_image.is_dev() {
            format!("{}-dev", self.server_type.short_name())
        } else {
            format!(
                "{}-{}",
                self.server_type.short_name(),
                self.wildfly_image.identifier
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
        if self.wildfly_image.is_dev() {
            format!("{}:{}", base_name, DEVELOPMENT_TAG)
        } else {
            format!(
                "{}:{}.{}",
                base_name, self.wildfly_image.version, self.wildfly_image.suffix
            )
        }
    }

    /// Returns the default container name (e.g. `"wado-sa-390"`).
    pub fn container_name(&self) -> String {
        format!("{}-{}", WILDFLY_ADMIN_CONTAINER, self.identifier())
    }
}

impl Ord for AdminImage {
    fn cmp(&self, other: &Self) -> Ordering {
        let a_dev = self.wildfly_image.is_dev();
        let b_dev = other.wildfly_image.is_dev();
        match (a_dev, b_dev) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => {
                let version_ord = self.wildfly_image.cmp(&other.wildfly_image);
                if version_ord == Ordering::Equal {
                    self.server_type.cmp(&other.server_type)
                } else {
                    version_ord
                }
            }
        }
    }
}

impl PartialOrd for AdminImage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wildfly_meta::parse_wildfly_image;

    fn test_registry() -> WildFlyImageRegistry {
        WildFlyImageRegistry::from_toml(include_str!("../../testdata/wildfly-images.toml"))
            .expect("failed to parse test registry")
    }

    fn wimg(version: &str) -> WildFlyImage {
        let registry = test_registry();
        parse_wildfly_image(version, &registry).unwrap()
    }

    #[test]
    fn new_sets_default_flags() {
        let ai = AdminImage::new(wimg("39"), ServerType::Standalone);
        assert!(!ai.local_image);
        assert!(!ai.in_use);
        assert_eq!(ai.server_type, ServerType::Standalone);
    }

    #[test]
    fn domain_creates_dc_and_hc() {
        let images = AdminImage::domain(wimg("39"));
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].server_type, ServerType::DomainController);
        assert_eq!(images[1].server_type, ServerType::HostController);
    }

    #[test]
    fn all_types_creates_three() {
        let images = AdminImage::all_types(wimg("39"));
        assert_eq!(images.len(), 3);
        assert_eq!(images[0].server_type, ServerType::Standalone);
        assert_eq!(images[1].server_type, ServerType::DomainController);
        assert_eq!(images[2].server_type, ServerType::HostController);
    }

    #[test]
    fn identifier_stable_version() {
        let ai = AdminImage::new(wimg("39"), ServerType::Standalone);
        assert_eq!(ai.identifier(), "sa-390");

        let ai = AdminImage::new(wimg("35"), ServerType::DomainController);
        assert_eq!(ai.identifier(), "dc-350");
    }

    #[test]
    fn identifier_dev_version() {
        let ai = AdminImage::new(wimg("dev"), ServerType::Standalone);
        assert_eq!(ai.identifier(), "sa-dev");
    }

    #[test]
    fn image_name_stable() {
        let ai = AdminImage::new(wimg("39"), ServerType::Standalone);
        let name = ai.image_name();
        assert!(name.starts_with("quay.io/wado/wado-sa:"));
        assert!(name.contains("39.0"));
    }

    #[test]
    fn image_name_dev() {
        let ai = AdminImage::new(wimg("dev"), ServerType::HostController);
        let name = ai.image_name();
        assert!(name.starts_with("quay.io/wado/wado-hc:"));
        assert!(name.contains(DEVELOPMENT_TAG));
    }

    #[test]
    fn container_name_format() {
        let ai = AdminImage::new(wimg("39"), ServerType::Standalone);
        assert_eq!(ai.container_name(), "wado-sa-390");

        let ai = AdminImage::new(wimg("dev"), ServerType::DomainController);
        assert_eq!(ai.container_name(), "wado-dc-dev");
    }

    #[test]
    fn from_identifier_valid_stable() {
        let registry = test_registry();
        let ai = AdminImage::from_identifier("sa-390".to_string(), &registry);
        assert!(ai.is_some());
        let ai = ai.unwrap();
        assert_eq!(ai.server_type, ServerType::Standalone);
        assert_eq!(ai.wildfly_image.identifier, 390);
    }

    #[test]
    fn from_identifier_valid_dev() {
        let registry = test_registry();
        let ai = AdminImage::from_identifier("dc-dev".to_string(), &registry);
        assert!(ai.is_some());
        let ai = ai.unwrap();
        assert_eq!(ai.server_type, ServerType::DomainController);
        assert!(ai.wildfly_image.is_dev());
    }

    #[test]
    fn from_identifier_invalid_no_dash() {
        let registry = test_registry();
        assert!(AdminImage::from_identifier("sa390".to_string(), &registry).is_none());
    }

    #[test]
    fn from_identifier_invalid_server_type() {
        let registry = test_registry();
        assert!(AdminImage::from_identifier("xx-390".to_string(), &registry).is_none());
    }

    #[test]
    fn from_identifier_invalid_version() {
        let registry = test_registry();
        assert!(AdminImage::from_identifier("sa-999".to_string(), &registry).is_none());
    }

    #[test]
    fn ordering_dev_after_stable() {
        let stable = AdminImage::new(wimg("39"), ServerType::Standalone);
        let dev = AdminImage::new(wimg("dev"), ServerType::Standalone);
        assert!(stable < dev);
    }

    #[test]
    fn ordering_same_version_by_server_type() {
        let sa = AdminImage::new(wimg("39"), ServerType::Standalone);
        let dc = AdminImage::new(wimg("39"), ServerType::DomainController);
        let hc = AdminImage::new(wimg("39"), ServerType::HostController);
        assert!(sa < dc);
        assert!(dc < hc);
    }

    #[test]
    fn ordering_by_version() {
        let v35 = AdminImage::new(wimg("35"), ServerType::Standalone);
        let v39 = AdminImage::new(wimg("39"), ServerType::Standalone);
        assert!(v35 < v39);
    }

    #[test]
    fn all_versions_by_image_name_not_empty() {
        let registry = test_registry();
        let map = AdminImage::all_versions_by_image_name(&registry);
        assert!(!map.is_empty());
        for (key, ai) in &map {
            assert_eq!(key, &ai.image_name());
        }
    }
}
