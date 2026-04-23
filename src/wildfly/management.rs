//! Management client configuration for JBoss CLI connections.

use semver::Version;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

use super::ContainerInstance;

/// Client configuration for connecting to a WildFly management interface via JBoss CLI.
pub struct ManagementClient {
    pub wildfly_container: WildFlyContainer,
    pub management_port: u16,
    pub cli_jar_url: String,
    pub cli_config_url: String,
}

impl ManagementClient {
    /// Creates a client using the version's default management port.
    pub fn default_port(wildfly_container: &WildFlyContainer) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(&wildfly_container.core_version);
        ManagementClient {
            wildfly_container: wildfly_container.clone(),
            management_port: wildfly_container.management_port(),
            cli_jar_url,
            cli_config_url,
        }
    }

    /// Creates a client with an explicit management port override.
    pub fn custom_port(
        wildfly_container: &WildFlyContainer,
        management_port: u16,
    ) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(&wildfly_container.core_version);
        ManagementClient {
            wildfly_container: wildfly_container.clone(),
            management_port,
            cli_jar_url,
            cli_config_url,
        }
    }

    /// Creates a client from a running container instance, using its actual port mappings.
    pub fn from_container_instance(container_instance: &ContainerInstance) -> ManagementClient {
        let management_port = if let Some(ports) = &container_instance.ports {
            ports.management
        } else {
            container_instance
                .admin_container
                .wildfly_container
                .management_port()
        };
        let (cli_jar_url, cli_config_url) = Self::urls(
            &container_instance
                .admin_container
                .wildfly_container
                .core_version,
        );
        ManagementClient {
            wildfly_container: container_instance.admin_container.wildfly_container.clone(),
            management_port,
            cli_jar_url,
            cli_config_url,
        }
    }

    fn urls(version: &Version) -> (String, String) {
        // Version(0,0,0) is the sentinel for dev builds - use the latest stable core version
        let effective_version = if *version == Version::new(0, 0, 0) {
            VERSIONS
                .last_key_value()
                .map(|(_, wfc)| wfc.core_version.clone())
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
    use crate::wildfly::{AdminContainer, ServerType};

    fn wc(version: &str) -> WildFlyContainer {
        WildFlyContainer::version(version).unwrap()
    }

    #[test]
    fn default_port_uses_version_management_port() {
        let wfc = wc("39");
        let client = ManagementClient::default_port(&wfc);
        assert_eq!(client.management_port, wfc.management_port());
    }

    #[test]
    fn custom_port_overrides_management_port() {
        let wfc = wc("39");
        let client = ManagementClient::custom_port(&wfc, 12345);
        assert_eq!(client.management_port, 12345);
    }

    #[test]
    fn urls_contain_version() {
        let wfc = wc("39");
        let client = ManagementClient::default_port(&wfc);
        assert!(client.cli_jar_url.contains(&wfc.core_version.to_string()));
        assert!(client
            .cli_config_url
            .contains(&wfc.core_version.to_string()));
    }

    #[test]
    fn urls_jar_ends_with_client_jar() {
        let wfc = wc("39");
        let client = ManagementClient::default_port(&wfc);
        assert!(client.cli_jar_url.ends_with("-client.jar"));
    }

    #[test]
    fn dev_version_uses_latest_stable_core() {
        let dev = wc("dev");
        let client = ManagementClient::default_port(&dev);
        assert!(!client.cli_jar_url.contains("0.0.0"));
        let latest = VERSIONS.last_key_value().unwrap().1;
        assert!(client
            .cli_jar_url
            .contains(&latest.core_version.to_string()));
    }

    #[test]
    fn from_container_instance_with_ports() {
        let ci = ContainerInstance::new("sa-390", "abc", "wado-sa-390", "Up", "", "").unwrap();
        let client = ManagementClient::from_container_instance(&ci);
        let expected_port = ci.ports.as_ref().unwrap().management;
        assert_eq!(client.management_port, expected_port);
    }

    #[test]
    fn from_container_instance_without_ports() {
        let ac = AdminContainer::new(wc("39"), ServerType::Standalone);
        let ci = ContainerInstance {
            admin_container: ac.clone(),
            running: true,
            container_id: "abc".to_string(),
            name: "test".to_string(),
            ports: None,
            status: "Up".to_string(),
            topology: None,
            config: None,
        };
        let client = ManagementClient::from_container_instance(&ci);
        assert_eq!(client.management_port, ac.wildfly_container.management_port());
    }
}
