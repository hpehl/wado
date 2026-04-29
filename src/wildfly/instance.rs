//! Container instance types for each WildFly server mode.
//!
//! Provides [`StandaloneInstance`], [`DomainController`], [`HostController`] for
//! containers about to be started, and [`ContainerInstance`] for running containers
//! parsed from `podman ps` output.

use crate::label::Label;
use anyhow::bail;
use std::cmp::Ordering;
use wildfly_meta::{WildFlyImage, WildFlyImageRegistry};

use super::AdminImage;

// ------------------------------------------------------ ports

/// HTTP and management port pair for a container instance.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ports {
    pub http: u16,
    pub management: u16,
}

impl Ports {
    /// Computes default ports from a WildFly image (HTTP: `8<major><minor>`, management: `9<major><minor>`).
    pub fn default_ports(wildfly_image: &WildFlyImage) -> Ports {
        Ports {
            http: wildfly_image.http_port(),
            management: wildfly_image.management_port(),
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
    pub admin_image: AdminImage,
    pub name: String,
    pub ports: Ports,
}

impl StandaloneInstance {
    pub fn new(admin_image: AdminImage, name: String, ports: Ports) -> StandaloneInstance {
        StandaloneInstance {
            admin_image,
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
    pub admin_image: AdminImage,
    pub name: String,
    pub ports: Ports,
}

impl DomainController {
    pub fn new(admin_image: AdminImage, name: String, ports: Ports) -> DomainController {
        DomainController {
            admin_image,
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
    pub admin_image: AdminImage,
    pub name: String,
    pub domain_controller: String,
}

impl HostController {
    pub fn new(admin_image: AdminImage, name: String, domain_controller: String) -> HostController {
        HostController {
            admin_image,
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
    pub admin_image: AdminImage,
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
        registry: &WildFlyImageRegistry,
    ) -> anyhow::Result<ContainerInstance> {
        if let Some(admin_image) = AdminImage::from_identifier(identifier.to_string(), registry) {
            let topology = Label::Topology.parse_value(topology);
            let config = Label::Config.parse_value(config);
            Ok(ContainerInstance {
                admin_image: admin_image.clone(),
                running: true,
                name: name.to_string(),
                container_id: container_id.to_string(),
                ports: Some(Ports::default_ports(&admin_image.wildfly_image)),
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
                self.admin_image
                    .cmp(&other.admin_image)
                    .then_with(|| self.name.cmp(&other.name))
            }),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self
                .admin_image
                .cmp(&other.admin_image)
                .then_with(|| self.name.cmp(&other.name)),
        }
    }
}

impl PartialOrd for ContainerInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::ServerType;
    use wildfly_meta::parse_image;

    fn test_registry() -> WildFlyImageRegistry {
        WildFlyImageRegistry::load_default().expect("failed to load image registry")
    }

    fn wimg(version: &str) -> WildFlyImage {
        let registry = test_registry();
        parse_image(version, &registry).unwrap()
    }

    #[test]
    fn default_ports_from_version() {
        let img = wimg("39");
        let ports = Ports::default_ports(&img);
        assert_eq!(ports.http, img.http_port());
        assert_eq!(ports.management, img.management_port());
    }

    #[test]
    fn ports_with_offset() {
        let ports = Ports {
            http: 8390,
            management: 9390,
        };
        let shifted = ports.with_offset(2);
        assert_eq!(shifted.http, 8392);
        assert_eq!(shifted.management, 9392);
    }

    #[test]
    fn standalone_instance_container_config() {
        use crate::wildfly::ContainerConfig;
        let ai = AdminImage::new(wimg("39"), ServerType::Standalone);
        let si = StandaloneInstance::new(
            ai.clone(),
            "my-server".to_string(),
            Ports::default_ports(&ai.wildfly_image),
        );
        assert_eq!(si.name(), "my-server");
        assert_eq!(si.admin_image().server_type, ServerType::Standalone);
    }

    #[test]
    fn domain_controller_container_config() {
        use crate::wildfly::ContainerConfig;
        let ai = AdminImage::new(wimg("39"), ServerType::DomainController);
        let dc = DomainController::new(
            ai.clone(),
            "dc-1".to_string(),
            Ports::default_ports(&ai.wildfly_image),
        );
        assert_eq!(dc.name(), "dc-1");
        assert_eq!(dc.admin_image().server_type, ServerType::DomainController);
    }

    #[test]
    fn host_controller_container_config() {
        use crate::wildfly::ContainerConfig;
        let ai = AdminImage::new(wimg("39"), ServerType::HostController);
        let hc = HostController::new(ai.clone(), "hc-1".to_string(), "dc-1".to_string());
        assert_eq!(hc.name(), "hc-1");
        assert_eq!(hc.domain_controller, "dc-1");
    }

    #[test]
    fn container_instance_new_valid() {
        let registry = test_registry();
        let ci = ContainerInstance::new(
            "sa-390",
            "abc123",
            "wado-sa-390",
            "Up 5 minutes",
            "",
            "",
            &registry,
        );
        assert!(ci.is_ok());
        let ci = ci.unwrap();
        assert_eq!(ci.name, "wado-sa-390");
        assert_eq!(ci.container_id, "abc123");
        assert!(ci.running);
        assert!(ci.topology.is_none());
        assert!(ci.config.is_none());
    }

    #[test]
    fn container_instance_new_with_labels() {
        let registry = test_registry();
        let ci = ContainerInstance::new(
            "dc-390",
            "def456",
            "wado-dc-390",
            "Up 10 minutes",
            "my-topo",
            "domain.xml",
            &registry,
        );
        assert!(ci.is_ok());
        let ci = ci.unwrap();
        assert_eq!(ci.topology, Some("my-topo".to_string()));
        assert_eq!(ci.config, Some("domain.xml".to_string()));
    }

    #[test]
    fn container_instance_new_invalid_identifier() {
        let registry = test_registry();
        let ci = ContainerInstance::new("xx-999", "abc", "name", "Up", "", "", &registry);
        assert!(ci.is_err());
    }

    #[test]
    fn container_instance_ordering_topology_before_no_topology() {
        let registry = test_registry();
        let with_topo =
            ContainerInstance::new("sa-390", "a", "a", "Up", "topo1", "", &registry).unwrap();
        let without_topo =
            ContainerInstance::new("sa-390", "b", "b", "Up", "", "", &registry).unwrap();
        assert!(with_topo < without_topo);
    }

    #[test]
    fn container_instance_ordering_same_topology_by_admin_image() {
        let registry = test_registry();
        let ci1 = ContainerInstance::new("sa-350", "a", "a", "Up", "topo", "", &registry).unwrap();
        let ci2 = ContainerInstance::new("sa-390", "b", "b", "Up", "topo", "", &registry).unwrap();
        assert!(ci1 < ci2);
    }

    #[test]
    fn container_instance_ordering_no_topology_by_name() {
        let registry = test_registry();
        let ci1 = ContainerInstance::new("sa-390", "a", "aaa", "Up", "", "", &registry).unwrap();
        let ci2 = ContainerInstance::new("sa-390", "b", "zzz", "Up", "", "", &registry).unwrap();
        assert!(ci1 < ci2);
    }
}
