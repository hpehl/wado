//! Management client configuration for JBoss CLI connections.

use semver::Version;
use wildfly_meta::{WildFlyImage, WildFlyImageRegistry};

use super::ContainerInstance;

/// Client configuration for connecting to a WildFly management interface via JBoss CLI.
pub struct ManagementClient {
    pub wildfly_image: WildFlyImage,
    pub management_port: u16,
    pub cli_jar_url: String,
    pub cli_config_url: String,
}

impl ManagementClient {
    /// Creates a client using the version's default management port.
    pub fn default_port(
        wildfly_image: &WildFlyImage,
        registry: &WildFlyImageRegistry,
    ) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(&wildfly_image.core_version, registry);
        ManagementClient {
            wildfly_image: wildfly_image.clone(),
            management_port: wildfly_image.management_port(),
            cli_jar_url,
            cli_config_url,
        }
    }

    /// Creates a client with an explicit management port override.
    pub fn custom_port(
        wildfly_image: &WildFlyImage,
        management_port: u16,
        registry: &WildFlyImageRegistry,
    ) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(&wildfly_image.core_version, registry);
        ManagementClient {
            wildfly_image: wildfly_image.clone(),
            management_port,
            cli_jar_url,
            cli_config_url,
        }
    }

    /// Creates a client from a running container instance, using its actual port mappings.
    pub fn from_container_instance(
        container_instance: &ContainerInstance,
        registry: &WildFlyImageRegistry,
    ) -> ManagementClient {
        let management_port = if let Some(ports) = &container_instance.ports {
            ports.management
        } else {
            container_instance
                .admin_image
                .wildfly_image
                .management_port()
        };
        let (cli_jar_url, cli_config_url) = Self::urls(
            &container_instance.admin_image.wildfly_image.core_version,
            registry,
        );
        ManagementClient {
            wildfly_image: container_instance.admin_image.wildfly_image.clone(),
            management_port,
            cli_jar_url,
            cli_config_url,
        }
    }

    fn urls(version: &Version, registry: &WildFlyImageRegistry) -> (String, String) {
        let effective_version = if *version == Version::new(0, 0, 0) {
            registry
                .last()
                .map(|img| img.core_version.clone())
                .unwrap_or_else(|| version.clone())
        } else {
            version.clone()
        };
        (
            format!(
                "https://repo1.maven.org/maven2/org/wildfly/core/wildfly-cli/{v}.Final/wildfly-cli-{v}.Final-client.jar",
                v = effective_version
            ),
            format!(
                "https://raw.githubusercontent.com/wildfly/wildfly-core/refs/tags/{}.Final/core-feature-pack/common/src/main/resources/content/bin/jboss-cli.xml",
                effective_version
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::{AdminImage, ServerType};
    use wildfly_meta::parse_wildfly_image;

    fn test_registry() -> WildFlyImageRegistry {
        WildFlyImageRegistry::load_default().expect("failed to load image registry")
    }

    fn wimg(version: &str) -> WildFlyImage {
        let registry = test_registry();
        parse_wildfly_image(version, &registry).unwrap()
    }

    #[test]
    fn default_port_uses_version_management_port() {
        let registry = test_registry();
        let img = wimg("39");
        let client = ManagementClient::default_port(&img, &registry);
        assert_eq!(client.management_port, img.management_port());
    }

    #[test]
    fn custom_port_overrides_management_port() {
        let registry = test_registry();
        let img = wimg("39");
        let client = ManagementClient::custom_port(&img, 12345, &registry);
        assert_eq!(client.management_port, 12345);
    }

    #[test]
    fn urls_contain_version() {
        let registry = test_registry();
        let img = wimg("39");
        let client = ManagementClient::default_port(&img, &registry);
        assert!(client.cli_jar_url.contains(&img.core_version.to_string()));
        assert!(
            client
                .cli_config_url
                .contains(&img.core_version.to_string())
        );
    }

    #[test]
    fn urls_jar_ends_with_client_jar() {
        let registry = test_registry();
        let img = wimg("39");
        let client = ManagementClient::default_port(&img, &registry);
        assert!(client.cli_jar_url.ends_with("-client.jar"));
    }

    #[test]
    fn dev_version_uses_latest_stable_core() {
        let registry = test_registry();
        let dev = wimg("dev");
        let client = ManagementClient::default_port(&dev, &registry);
        assert!(!client.cli_jar_url.contains("0.0.0"));
        let latest = registry.last().unwrap();
        assert!(
            client
                .cli_jar_url
                .contains(&latest.core_version.to_string())
        );
    }

    #[test]
    fn from_container_instance_with_ports() {
        let registry = test_registry();
        let ci = ContainerInstance::new("sa-390", "abc", "wado-sa-390", "Up", "", "", &registry)
            .unwrap();
        let client = ManagementClient::from_container_instance(&ci, &registry);
        let expected_port = ci.ports.as_ref().unwrap().management;
        assert_eq!(client.management_port, expected_port);
    }

    #[test]
    fn from_container_instance_without_ports() {
        let registry = test_registry();
        let img = wimg("39");
        let ai = AdminImage::new(img.clone(), ServerType::Standalone);
        let ci = ContainerInstance {
            admin_image: ai.clone(),
            running: true,
            container_id: "abc".to_string(),
            name: "test".to_string(),
            ports: None,
            status: "Up".to_string(),
            topology: None,
            config: None,
        };
        let client = ManagementClient::from_container_instance(&ci, &registry);
        assert_eq!(client.management_port, ai.wildfly_image.management_port());
    }
}
