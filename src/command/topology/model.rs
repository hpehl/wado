use crate::wildfly::{Server, ServerGroup};
use anyhow::{Context, bail};
use serde::de;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use wildfly_container_versions::WildFlyContainer;

#[derive(Deserialize)]
pub struct TopologySetup {
    pub name: String,
    #[serde(deserialize_with = "deserialize_version")]
    pub version: String,
    pub hosts: Vec<HostSetup>,
}

#[derive(Deserialize)]
pub struct HostSetup {
    pub name: Option<String>,
    #[serde(rename = "domain-controller", default)]
    pub domain_controller: bool,
    #[serde(default, deserialize_with = "deserialize_optional_version")]
    pub version: Option<String>,
    #[serde(default)]
    pub servers: Vec<ServerSetup>,
}

#[derive(Deserialize)]
pub struct ServerSetup {
    pub name: String,
    pub group: Option<String>,
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
        resolve_version(&setup.version)
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
                .map(|h| h.name.as_deref().unwrap_or("<unnamed>"))
                .collect();
            bail!("Multiple domain controllers defined: {}", names.join(", "));
        }

        let mut seen = HashSet::new();
        for host in &self.hosts {
            if let Some(name) = &host.name
                && !seen.insert(name)
            {
                bail!("Duplicate host name: '{}'", name);
            }
        }

        let is_dev = self.version == "dev";
        for host in &self.hosts {
            let host_label = host.name.as_deref().unwrap_or("<unnamed>");
            if let Some(v) = &host.version {
                let host_is_dev = v == "dev";
                if is_dev && !host_is_dev {
                    bail!(
                        "Cannot mix dev and versioned hosts. \
                         Top-level version is 'dev', but host '{}' uses version '{}'",
                        host_label,
                        v
                    );
                }
                if !is_dev && host_is_dev {
                    bail!(
                        "Cannot mix dev and versioned hosts. \
                         Top-level version is '{}', but host '{}' uses version 'dev'",
                        self.version,
                        host_label
                    );
                }
                resolve_version(v).with_context(|| {
                    format!("Unknown WildFly version '{}' for host '{}'", v, host_label)
                })?;
            }
            for server in &host.servers {
                if let Some(group) = &server.group
                    && ServerGroup::parse_group(group).is_none()
                {
                    bail!(
                        "Invalid server group '{}' for server '{}' on host '{}'",
                        group,
                        server.name,
                        host_label
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
}

impl HostSetup {
    pub fn effective_version<'a>(&'a self, default: &'a str) -> &'a str {
        self.version.as_deref().unwrap_or(default)
    }
}

fn resolve_version(version: &str) -> anyhow::Result<WildFlyContainer> {
    WildFlyContainer::version(version).map_err(|e| anyhow::anyhow!("{}", e))
}

struct VersionVisitor;

impl<'de> de::Visitor<'de> for VersionVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a version number (e.g. 34, 26.1) or 'dev'")
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
        Ok(v.to_string())
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
        Ok(format!("{v}"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
        Ok(v.to_string())
    }
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_any(VersionVisitor)
}

fn deserialize_optional_version<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    Option::<serde_yml::Value>::deserialize(deserializer)?
        .map(|v| match v {
            serde_yml::Value::Number(n) => Ok(n.to_string()),
            serde_yml::Value::String(s) => Ok(s),
            _ => Err(de::Error::custom("expected a version number or 'dev'")),
        })
        .transpose()
}

impl ServerSetup {
    pub fn to_server(&self) -> Server {
        Server {
            name: self.name.clone(),
            server_group: self
                .group
                .as_deref()
                .and_then(ServerGroup::parse_group)
                .unwrap_or(ServerGroup::MainServerGroup),
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
name: test-topology
version: 34
hosts:
  - name: dc
    domain-controller: true
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.name, "test-topology");
        assert_eq!(setup.version, "34");
        assert_eq!(setup.hosts.len(), 1);
        assert!(setup.hosts[0].domain_controller);
        assert_eq!(setup.hosts[0].name, Some("dc".to_string()));
        assert!(setup.hosts[0].servers.is_empty());
    }

    #[test]
    fn deserialize_dev_version() {
        let yaml = r#"
name: dev-topology
version: dev
hosts:
  - name: dc
    domain-controller: true
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.version, "dev");
    }

    #[test]
    fn deserialize_dotted_version() {
        let yaml = r#"
name: test-topology
version: 26.1
hosts:
  - name: dc
    domain-controller: true
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.version, "26.1");
    }

    #[test]
    fn deserialize_unnamed_hosts() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - domain-controller: true
  - servers:
      - name: server-one
        group: main-server-group
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.hosts.len(), 2);
        assert!(setup.hosts[0].name.is_none());
        assert!(setup.hosts[1].name.is_none());
    }

    #[test]
    fn deserialize_full() {
        let yaml = r#"
name: test-topology
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
        assert_eq!(setup.dc_host().name, Some("dc".to_string()));

        let host1 = &setup.hosts[1];
        assert!(!host1.domain_controller);
        assert_eq!(host1.servers.len(), 3);
        assert!(host1.servers[0].auto_start);
        assert_eq!(host1.servers[1].offset, 100);

        let host2 = &setup.hosts[2];
        assert_eq!(host2.version, Some("33".to_string()));
        assert_eq!(host2.effective_version("34"), "33");
    }

    #[test]
    fn deserialize_version_override() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: 33
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let host1 = &setup.hosts[1];
        assert_eq!(host1.effective_version("34"), "33");
        assert_eq!(setup.dc_host().effective_version("34"), "34");
    }

    #[test]
    fn deserialize_server_without_group() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - domain-controller: true
  - servers:
      - name: server-one
      - name: server-two
        group: other-server-group
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert!(setup.validate().is_ok());
        let servers = &setup.hosts[1].servers;
        assert!(servers[0].group.is_none());
        assert_eq!(
            servers[0].to_server().server_group,
            ServerGroup::MainServerGroup
        );
        assert_eq!(
            servers[1].to_server().server_group,
            ServerGroup::OtherServerGroup
        );
    }

    #[test]
    fn validate_no_dc() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - name: host1
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No domain controller")
        );
    }

    #[test]
    fn validate_multiple_dcs() {
        let yaml = r#"
name: test-topology
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Multiple domain controllers")
        );
    }

    #[test]
    fn validate_duplicate_names() {
        let yaml = r#"
name: test-topology
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate host name")
        );
    }

    #[test]
    fn validate_invalid_server_group() {
        let yaml = r#"
name: test-topology
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid server group")
        );
    }

    #[test]
    fn validate_unnamed_no_duplicate_error() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - domain-controller: true
  - servers:
      - name: server-one
        group: main-server-group
  - servers:
      - name: server-two
        group: other-server-group
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert!(setup.validate().is_ok());
    }

    #[test]
    fn validate_mixed_dev_and_stable() {
        let yaml = r#"
name: test-topology
version: dev
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: 34
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot mix dev"));
    }

    #[test]
    fn validate_mixed_stable_and_dev() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: dev
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        let result = setup.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot mix dev"));
    }

    #[test]
    fn deserialize_host_dev_version_override() {
        let yaml = r#"
name: dev-topology
version: dev
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: dev
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert!(setup.validate().is_ok());
        assert_eq!(setup.hosts[1].version, Some("dev".to_string()));
        assert_eq!(setup.hosts[1].effective_version("dev"), "dev");
    }

    #[test]
    fn deserialize_host_dotted_version_override() {
        let yaml = r#"
name: test-topology
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    version: 26.1
"#;
        let setup: TopologySetup = serde_yml::from_str(yaml).unwrap();
        assert_eq!(setup.hosts[1].version, Some("26.1".to_string()));
        assert_eq!(setup.hosts[1].effective_version("34"), "26.1");
    }

    #[test]
    fn server_setup_to_server() {
        let setup = ServerSetup {
            name: "server-one".to_string(),
            group: Some("main-server-group".to_string()),
            offset: 100,
            auto_start: true,
        };
        let server = setup.to_server();
        assert_eq!(server.name, "server-one");
        assert_eq!(server.server_group, ServerGroup::MainServerGroup);
        assert_eq!(server.offset, 100);
        assert!(server.autostart);
    }

    #[test]
    fn server_setup_to_server_osg() {
        let setup = ServerSetup {
            name: "server-two".to_string(),
            group: Some("other-server-group".to_string()),
            offset: 200,
            auto_start: false,
        };
        let server = setup.to_server();
        assert_eq!(server.server_group, ServerGroup::OtherServerGroup);
        assert_eq!(server.offset, 200);
        assert!(!server.autostart);
    }

    #[test]
    fn server_setup_to_server_default_group() {
        let setup = ServerSetup {
            name: "server-three".to_string(),
            group: None,
            offset: 0,
            auto_start: false,
        };
        let server = setup.to_server();
        assert_eq!(server.server_group, ServerGroup::MainServerGroup);
    }
}
