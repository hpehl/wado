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
