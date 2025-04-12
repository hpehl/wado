use crate::constants::{WILDFLY_ADMIN_CONTAINER, WILDFLY_ADMIN_CONTAINER_REPOSITORY};
use anyhow::bail;
use semver::Version;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ traits

pub trait HasWildFlyContainer {
    fn wildfly_container(&self) -> &WildFlyContainer;
}

// ------------------------------------------------------ server type

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServerType {
    Standalone,
    DomainController,
    HostController,
}

impl ServerType {
    pub fn short_name(&self) -> &'static str {
        match self {
            ServerType::Standalone => "sa",
            ServerType::DomainController => "dc",
            ServerType::HostController => "hc",
        }
    }
}

impl FromStr for ServerType {
    type Err = ();

    fn from_str(input: &str) -> Result<ServerType, Self::Err> {
        match input {
            "sa" => Ok(ServerType::Standalone),
            "dc" => Ok(ServerType::DomainController),
            "hc" => Ok(ServerType::HostController),
            _ => Err(()),
        }
    }
}

// ------------------------------------------------------ admin container

#[derive(Eq, PartialEq, Clone)]
pub struct AdminContainer {
    pub wildfly_container: WildFlyContainer,
    pub server_type: ServerType,
}

impl AdminContainer {
    pub fn standalone(wildfly_container: WildFlyContainer) -> AdminContainer {
        AdminContainer {
            wildfly_container,
            server_type: ServerType::Standalone,
        }
    }

    pub fn domain_controller(wildfly_container: WildFlyContainer) -> AdminContainer {
        AdminContainer {
            wildfly_container,
            server_type: ServerType::DomainController,
        }
    }

    pub fn host_controller(wildfly_container: WildFlyContainer) -> AdminContainer {
        AdminContainer {
            wildfly_container,
            server_type: ServerType::HostController,
        }
    }

    pub fn domain(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::domain_controller(wildfly_container.clone()),
            AdminContainer::host_controller(wildfly_container.clone()),
        ]
    }

    pub fn all_types(wildfly_container: WildFlyContainer) -> Vec<AdminContainer> {
        vec![
            AdminContainer::standalone(wildfly_container.clone()),
            AdminContainer::domain_controller(wildfly_container.clone()),
            AdminContainer::host_controller(wildfly_container.clone()),
        ]
    }

    pub fn from_identifier(identifier: String) -> Option<AdminContainer> {
        // TODO Support 'dev' identifier
        if identifier.contains('-') {
            let parts = identifier.split('-').collect::<Vec<&str>>();
            if parts.len() == 2 {
                if let Ok(identifier) = parts[0].parse::<u16>() {
                    if let Ok(wildfly_container) = WildFlyContainer::lookup(identifier) {
                        if let Ok(server_type) = ServerType::from_str(parts[1]) {
                            return Some(AdminContainer {
                                wildfly_container,
                                server_type,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    pub fn identifier(&self) -> String {
        format!(
            "{}-{}",
            self.wildfly_container.identifier,
            self.server_type.short_name()
        )
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
        if self.wildfly_container == other.wildfly_container {
            self.server_type.cmp(&other.server_type)
        } else {
            self.wildfly_container.cmp(&other.wildfly_container)
        }
    }
}

impl PartialOrd for AdminContainer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HasWildFlyContainer for AdminContainer {
    fn wildfly_container(&self) -> &WildFlyContainer {
        &self.wildfly_container
    }
}

// ------------------------------------------------------ ports

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ports {
    pub http: u16,
    pub management: u16,
}

impl Ports {
    pub fn default_ports(wildfly_container: &WildFlyContainer) -> Ports {
        Ports {
            http: wildfly_container.http_port(),
            management: wildfly_container.management_port(),
        }
    }

    pub fn with_offset(&self, offset: u16) -> Ports {
        Ports {
            http: self.http + offset,
            management: self.management + offset,
        }
    }
}

// ------------------------------------------------------ standalone instance

#[derive(Clone)]
pub struct StandaloneInstance {
    pub admin_container: AdminContainer,
    pub name: String,
    pub ports: Ports,
}

impl StandaloneInstance {
    pub fn new(admin_container: AdminContainer, name: String, ports: Ports) -> StandaloneInstance {
        StandaloneInstance {
            admin_container,
            name,
            ports,
        }
    }

    pub fn copy(&self, index: u16) -> StandaloneInstance {
        StandaloneInstance {
            admin_container: self.admin_container.clone(),
            name: format!("{}-{}", self.name, index),
            ports: self.ports.with_offset(index),
        }
    }
}

impl HasWildFlyContainer for StandaloneInstance {
    fn wildfly_container(&self) -> &WildFlyContainer {
        &self.admin_container.wildfly_container
    }
}

// ------------------------------------------------------ domain controller

#[derive(Clone)]
pub struct DomainController {
    pub admin_container: AdminContainer,
    pub name: String,
    pub ports: Ports,
}

impl DomainController {
    pub fn new(admin_container: AdminContainer, name: String, ports: Ports) -> DomainController {
        DomainController {
            admin_container,
            name,
            ports,
        }
    }

    pub fn copy(&self, index: u16) -> DomainController {
        DomainController {
            admin_container: self.admin_container.clone(),
            name: format!("{}-{}", self.name, index),
            ports: self.ports.with_offset(index),
        }
    }
}

impl HasWildFlyContainer for DomainController {
    fn wildfly_container(&self) -> &WildFlyContainer {
        &self.admin_container.wildfly_container
    }
}

// ------------------------------------------------------ host controller

#[derive(Clone)]
pub struct HostController {
    pub admin_container: AdminContainer,
    pub name: String,
    pub domain_controller: String,
}

impl HostController {
    pub fn new(
        admin_container: AdminContainer,
        name: String,
        domain_controller: String,
    ) -> HostController {
        HostController {
            admin_container,
            name,
            domain_controller,
        }
    }

    pub fn copy(&self, index: u16) -> HostController {
        HostController {
            admin_container: self.admin_container.clone(),
            name: format!("{}-{}", self.name, index),
            domain_controller: self.domain_controller.clone(),
        }
    }
}

impl HasWildFlyContainer for HostController {
    fn wildfly_container(&self) -> &WildFlyContainer {
        &self.admin_container.wildfly_container
    }
}

// ------------------------------------------------------ container instance

#[derive(Eq, PartialEq, Clone)]
pub struct ContainerInstance {
    pub admin_container: AdminContainer,
    pub running: bool,
    pub container_id: String,
    pub name: String,
    pub ports: Option<Ports>,
    pub status: String,
}

impl ContainerInstance {
    pub fn new(
        identifier: &str,
        container_id: &str,
        name: &str,
        status: &str,
    ) -> anyhow::Result<ContainerInstance> {
        if let Some(admin_container) = AdminContainer::from_identifier(identifier.to_string()) {
            Ok(ContainerInstance {
                admin_container: admin_container.clone(),
                running: true,
                name: name.to_string(),
                container_id: container_id.to_string(),
                ports: Some(Ports::default_ports(&admin_container.wildfly_container)),
                status: status.to_string(),
            })
        } else {
            bail!("Invalid identifier: '{}'", identifier);
        }
    }
}

impl Ord for ContainerInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.admin_container == other.admin_container {
            self.name.cmp(&other.name)
        } else {
            self.admin_container.cmp(&other.admin_container)
        }
    }
}

impl PartialOrd for ContainerInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ------------------------------------------------------ server

#[derive(Clone, Debug, PartialEq)]
pub enum ServerGroup {
    MainServerGroup,
    OtherServerGroup,
}

impl Display for ServerGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerGroup::MainServerGroup => write!(f, "main-server-group"),
            ServerGroup::OtherServerGroup => write!(f, "other-server-group"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Server {
    pub name: String,
    pub server_group: ServerGroup,
    pub offset: u16,
    pub autostart: bool,
}

impl Server {
    pub fn parse_servers(input: &str) -> anyhow::Result<Vec<Server>> {
        input.split(',').map(Server::parse_server).collect()
    }

    pub fn parse_server(input: &str) -> anyhow::Result<Server> {
        let mut parts = input.split(':').collect::<Vec<&str>>();

        if parts.is_empty() {
            bail!("Invalid input format");
        }

        let name = parts.remove(0).to_string();
        if name.is_empty() {
            bail!("Invalid input format");
        }

        let mut server_group = ServerGroup::MainServerGroup;
        let mut offset = 0;
        let mut autostart = false;

        if !parts.is_empty() {
            if parts[0].eq_ignore_ascii_case("start") {
                autostart = true;
                parts.remove(0);
            } else if let Ok(o) = parts[0].parse::<u16>() {
                offset = o;
                parts.remove(0);

                if !parts.is_empty() && parts[0].eq_ignore_ascii_case("start") {
                    autostart = true;
                    parts.remove(0);
                } else if !parts.is_empty() {
                    bail!("Invalid input format");
                }
            } else {
                let server_group_value = parts.remove(0).to_string();
                if !server_group_value.is_empty() {
                    if server_group_value.eq_ignore_ascii_case("msg")
                        || server_group_value.eq_ignore_ascii_case("main-server-group")
                    {
                        server_group = ServerGroup::MainServerGroup;
                    } else if server_group_value.eq_ignore_ascii_case("osg")
                        || server_group_value.eq_ignore_ascii_case("other-server-group")
                    {
                        server_group = ServerGroup::OtherServerGroup;
                    } else {
                        bail!("Invalid server group: '{}'", server_group_value);
                    }
                }

                if !parts.is_empty() {
                    if parts[0].eq_ignore_ascii_case("start") {
                        autostart = true;
                        parts.remove(0);
                    } else if let Ok(o) = parts[0].parse::<u16>() {
                        offset = o;
                        parts.remove(0);

                        if !parts.is_empty() && parts[0].eq_ignore_ascii_case("start") {
                            autostart = true;
                            parts.remove(0);
                        }
                    } else if parts[0].eq_ignore_ascii_case("start") {
                        autostart = true;
                        parts.remove(0);
                    } else {
                        bail!("Invalid input format".to_string());
                    }
                }
            }
        }

        Ok(Server {
            name,
            server_group,
            offset,
            autostart,
        })
    }

    pub fn with_offset(&self, offset: u16) -> Server {
        Server {
            name: self.name.clone(),
            server_group: self.server_group.clone(),
            offset,
            autostart: self.autostart,
        }
    }

    pub fn add_server_op(&self, host: &str) -> String {
        if self.offset > 0 {
            format!(
                "/host={}/server-config={}:add(group={},socket-binding-port-offset={},auto-start={})",
                host, self.name, self.server_group, self.offset, self.autostart
            )
        } else {
            format!(
                "/host={}/server-config={}:add(group={},auto-start={})",
                host, self.name, self.server_group, self.autostart
            )
        }
    }
}

// ------------------------------------------------------ management client

pub struct ManagementClient {
    pub wildfly_container: WildFlyContainer,
    pub management_port: u16,
    pub cli_jar_url: String,
    pub cli_config_url: String,
}

impl ManagementClient {
    pub fn default_port(wildfly_container: &WildFlyContainer) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(&wildfly_container.core_version);
        ManagementClient {
            wildfly_container: wildfly_container.clone(),
            management_port: wildfly_container.management_port(),
            cli_jar_url,
            cli_config_url,
        }
    }

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

    pub fn from_container_instance(container_instance: &ContainerInstance) -> ManagementClient {
        let (cli_jar_url, cli_config_url) = Self::urls(
            &container_instance
                .admin_container
                .wildfly_container
                .core_version,
        );
        ManagementClient {
            wildfly_container: container_instance.admin_container.wildfly_container.clone(),
            management_port: container_instance
                .admin_container
                .wildfly_container
                .management_port(),
            cli_jar_url,
            cli_config_url,
        }
    }

    fn urls(version: &Version) -> (String, String) {
        (
            format!(
                "https://repo1.maven.org/maven2/org/wildfly/core/wildfly-cli/{v}.Final/wildfly-cli-{v}.Final-client.jar",
                v = version
            ),
            format!(
                "https://raw.githubusercontent.com/wildfly/wildfly-core/refs/tags/{}.Final/core-feature-pack/common/src/main/resources/content/bin/jboss-cli.xml",
                version
            ),
        )
    }
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::ServerGroup::{MainServerGroup, OtherServerGroup};

    #[test]
    fn parse_server_name_only() {
        let result = Server::parse_server("server1").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_server_group() {
        let result = Server::parse_server("server1:msg").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:main-server-group").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:osg").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, OtherServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:other-server-group").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, OtherServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_offset() {
        let result = Server::parse_server("server1:123").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_autostart() {
        let result = Server::parse_server("server1:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_offset() {
        let result = Server::parse_server("server1:msg:123").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_autostart() {
        let result = Server::parse_server("server1:msg:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_offset_autostart() {
        let result = Server::parse_server("server1:123:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_offset_autostart() {
        let result = Server::parse_server("server1:msg:123:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_invalid() {
        let result = Server::parse_server("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_invalid_server_group() {
        let result = Server::parse_server("server1:groupA");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_invalid_offset() {
        let result = Server::parse_server("server1:msg:abc:start");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_offset_before_group() {
        let result = Server::parse_server("server1:123:groupA:start");
        assert!(result.is_err());
    }

    #[test]
    fn add_server_op_no_offset() {
        let server = Server {
            name: "server-one".to_string(),
            server_group: MainServerGroup,
            offset: 0,
            autostart: true,
        };
        assert_eq!(
            server.add_server_op("primary"),
            "/host=primary/server-config=server-one:add(group=main-server-group,auto-start=true)"
        );
    }

    #[test]
    fn add_server_op_with_offset() {
        let server = Server {
            name: "server-two".to_string(),
            server_group: OtherServerGroup,
            offset: 100,
            autostart: false,
        };
        assert_eq!(
            server.add_server_op("secondary"),
            "/host=secondary/server-config=server-two:add(group=other-server-group,socket-binding-port-offset=100,auto-start=false)"
        );
    }
}
