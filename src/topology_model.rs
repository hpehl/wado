use crate::wildfly::{Server, ServerGroup};
use anyhow::{bail, Context};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use wildfly_container_versions::WildFlyContainer;

#[derive(Deserialize)]
pub struct TopologySetup {
    pub version: u16,
    pub hosts: Vec<HostSetup>,
}

#[derive(Deserialize)]
pub struct HostSetup {
    pub name: String,
    #[serde(rename = "domain-controller", default)]
    pub domain_controller: bool,
    pub version: Option<u16>,
    #[serde(default)]
    pub servers: Vec<ServerSetup>,
}

#[derive(Deserialize)]
pub struct ServerSetup {
    pub name: String,
    pub group: String,
    #[serde(default)]
    pub offset: u16,
    #[serde(rename = "auto-start", default)]
    pub auto_start: bool,
}

impl TopologySetup {
    pub fn load(path: &Path) -> anyhow::Result<TopologySetup> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read topology file: {}", path.display()))?;
        let setup: TopologySetup = serde_yml::from_str(&content)
            .with_context(|| format!("Failed to parse topology file: {}", path.display()))?;
        setup.validate()?;
        setup
            .resolve_version(setup.version)
            .with_context(|| format!("Unknown WildFly version: {}", setup.version))?;
        Ok(setup)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let dc_count = self.hosts.iter().filter(|h| h.domain_controller).count();
        if dc_count == 0 {
            bail!("No domain controller defined in topology");
        }
        if dc_count > 1 {
            let names: Vec<&str> = self
                .hosts
                .iter()
                .filter(|h| h.domain_controller)
                .map(|h| h.name.as_str())
                .collect();
            bail!("Multiple domain controllers defined: {}", names.join(", "));
        }

        let mut seen = HashSet::new();
        for host in &self.hosts {
            if !seen.insert(&host.name) {
                bail!("Duplicate host name: '{}'", host.name);
            }
        }

        for host in &self.hosts {
            if let Some(v) = host.version {
                self.resolve_version(v)
                    .with_context(|| format!("Unknown WildFly version {} for host '{}'", v, host.name))?;
            }
            for server in &host.servers {
                if ServerGroup::parse_group(&server.group).is_none() {
                    bail!(
                        "Invalid server group '{}' for server '{}' on host '{}'",
                        server.group,
                        server.name,
                        host.name
                    );
                }
            }
        }
        Ok(())
    }

    pub fn dc_host(&self) -> &HostSetup {
        self.hosts
            .iter()
            .find(|h| h.domain_controller)
            .expect("No domain controller found (should have been validated)")
    }

    pub fn hc_hosts(&self) -> Vec<&HostSetup> {
        self.hosts.iter().filter(|h| !h.domain_controller).collect()
    }

    fn resolve_version(&self, version: u16) -> anyhow::Result<WildFlyContainer> {
        WildFlyContainer::version(&version.to_string())
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl HostSetup {
    pub fn effective_version(&self, default: u16) -> u16 {
        self.version.unwrap_or(default)
    }
}

impl ServerSetup {
    pub fn to_server(&self) -> Server {
        Server {
            name: self.name.clone(),
            server_group: ServerGroup::parse_group(&self.group)
                .expect("Invalid server group (should have been validated)"),
            offset: self.offset,
            autostart: self.auto_start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal() {
        let yaml = r#"
version: 34
hosts:
  - name: dc
    domain-controller: true
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.version, 34);
        assert_eq!(setup.hosts.len(), 1);
        assert!(setup.hosts[0].domain_controller);
        assert_eq!(setup.hosts[0].name, "dc");
        assert!(setup.hosts[0].servers.is_empty());
    }

    #[test]
    fn deserialize_full() {
        let yaml = r#"
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    servers:
      - name: server-one
        group: main-server-group
        auto-start: true
      - name: server-two
        group: main-server-group
        offset: 100
      - name: server-three
        group: other-server-group
        offset: 200
  - name: host2
    version: 33
    servers:
      - name: server-one
        group: main-server-group
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.hosts.len(), 3);
        assert_eq!(setup.hc_hosts().len(), 2);
        assert_eq!(setup.dc_host().name, "dc");

        let host1 = &setup.hosts[1];
        assert!(!host1.domain_controller);
        assert_eq!(host1.servers.len(), 3);
        assert!(host1.servers[0].auto_start);
        assert_eq!(host1.servers[1].offset, 100);

        let host2 = &setup.hosts[2];
        assert_eq!(host2.version, Some(33));
        assert_eq!(host2.effective_version(34), 33);
    }

    #[test]
    fn deserialize_version_override() {
        let yaml = r#"
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: 33
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let host1 = &setup.hosts[1];
        assert_eq!(host1.effective_version(34), 33);
        assert_eq!(setup.dc_host().effective_version(34), 34);
    }

    #[test]
    fn validate_no_dc() {
        let yaml = r#"
version: 34
hosts:
  - name: host1
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No domain controller"));
    }

    #[test]
    fn validate_multiple_dcs() {
        let yaml = r#"
version: 34
hosts:
  - name: dc1
    domain-controller: true
  - name: dc2
    domain-controller: true
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple domain controllers"));
    }

    #[test]
    fn validate_duplicate_names() {
        let yaml = r#"
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
  - name: host1
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate host name"));
    }

    #[test]
    fn validate_invalid_server_group() {
        let yaml = r#"
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    servers:
      - name: server-one
        group: invalid-group
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid server group"));
    }

    #[test]
    fn server_setup_to_server() {
        let setup = ServerSetup {
            name: "server-one".to_string(),
            group: "main-server-group".to_string(),
            offset: 100,
            auto_start: true,
        };
        let server = setup.to_server();
        assert_eq!(server.name, "server-one");
        assert_eq!(server.server_group, crate::wildfly::ServerGroup::MainServerGroup);
        assert_eq!(server.offset, 100);
        assert!(server.autostart);
    }

    #[test]
    fn server_setup_to_server_osg() {
        let setup = ServerSetup {
            name: "server-two".to_string(),
            group: "other-server-group".to_string(),
            offset: 200,
            auto_start: false,
        };
        let server = setup.to_server();
        assert_eq!(server.server_group, crate::wildfly::ServerGroup::OtherServerGroup);
        assert_eq!(server.offset, 200);
        assert!(!server.autostart);
    }
}
