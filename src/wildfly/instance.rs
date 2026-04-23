//! Container instance types for each WildFly server mode.
//!
//! Provides [`StandaloneInstance`], [`DomainController`], [`HostController`] for
//! containers about to be started, and [`ContainerInstance`] for running containers
//! parsed from `podman ps` output.

use crate::label::Label;
use anyhow::bail;
use std::cmp::Ordering;
use wildfly_container_versions::WildFlyContainer;

use super::AdminContainer;

// ------------------------------------------------------ ports

/// HTTP and management port pair for a container instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ports {
    pub http: u16,
    pub management: u16,
}

impl Ports {
    /// Computes default ports from a WildFly version (HTTP: `8<major><minor>`, management: `9<major><minor>`).
    pub fn default_ports(wildfly_container: &WildFlyContainer) -> Ports {
        Ports {
            http: wildfly_container.http_port(),
            management: wildfly_container.management_port(),
        }
    }

    #[cfg(test)]
    pub fn with_offset(&self, offset: u16) -> Ports {
        Ports {
            http: self.http + offset,
            management: self.management + offset,
        }
    }
}

// ------------------------------------------------------ standalone instance

/// A standalone WildFly server instance with its own HTTP and management ports.
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
}

impl_container_instance!(StandaloneInstance);

// ------------------------------------------------------ domain controller

/// A WildFly domain controller instance managing host controllers in a domain.
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
}

impl_container_instance!(DomainController);

// ------------------------------------------------------ host controller

/// A WildFly host controller instance connected to a domain controller.
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
}

impl_container_instance!(HostController);

// ------------------------------------------------------ container instance

/// A running container instance parsed from `podman ps` output.
#[derive(Eq, PartialEq, Clone)]
pub struct ContainerInstance {
    pub admin_container: AdminContainer,
    pub running: bool,
    pub container_id: String,
    pub name: String,
    pub ports: Option<Ports>,
    pub status: String,
    pub topology: Option<String>,
    pub config: Option<String>,
}

impl ContainerInstance {
    /// Parses a running container from `podman ps` output fields.
    pub fn new(
        identifier: &str,
        container_id: &str,
        name: &str,
        status: &str,
        topology: &str,
        config: &str,
    ) -> anyhow::Result<ContainerInstance> {
        if let Some(admin_container) = AdminContainer::from_identifier(identifier.to_string()) {
            let topology = Label::Topology.parse_value(topology);
            let config = Label::Config.parse_value(config);
            Ok(ContainerInstance {
                admin_container: admin_container.clone(),
                running: true,
                name: name.to_string(),
                container_id: container_id.to_string(),
                ports: Some(Ports::default_ports(&admin_container.wildfly_container)),
                status: status.to_string(),
                topology,
                config,
            })
        } else {
            bail!("Invalid identifier: '{}'", identifier);
        }
    }
}

impl Ord for ContainerInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.topology, &other.topology) {
            (Some(a), Some(b)) => a.cmp(b).then_with(|| {
                self.admin_container
                    .cmp(&other.admin_container)
                    .then_with(|| self.name.cmp(&other.name))
            }),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self
                .admin_container
                .cmp(&other.admin_container)
                .then_with(|| self.name.cmp(&other.name)),
        }
    }
}

impl PartialOrd for ContainerInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
