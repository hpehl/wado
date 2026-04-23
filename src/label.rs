//! OCI container label helpers for wado container metadata.
//!
//! Wado uses custom labels to tag containers with their identity, topology
//! membership, and configuration. This module provides helpers for constructing
//! the label keys, filter expressions, and format templates used with `podman`.

/// OCI label types used to annotate wado containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    /// Container identifier (e.g. `"sa-390"`), used to match containers to admin container metadata.
    Id,
    /// Topology name, set when the container was started as part of a topology.
    Topology,
    /// Server configuration file name (e.g. `"standalone.xml"`).
    Config,
}

impl Label {
    /// Returns the fully qualified OCI label key (e.g. `"org.wildfly.wado.id"`).
    pub fn key(&self) -> &'static str {
        match self {
            Label::Id => "org.wildfly.wado.id",
            Label::Topology => "org.wildfly.wado.topology",
            Label::Config => "org.wildfly.wado.config",
        }
    }

    /// For `podman ps --filter label=<key>` (existence check)
    pub fn filter(&self) -> String {
        format!("label={}", self.key())
    }

    /// For `podman ps --filter label=<key>=<value>` (exact match)
    pub fn filter_value(&self, value: &str) -> String {
        format!("label={}={}", self.key(), value)
    }

    /// For `podman run --label <key>=<value>`
    pub fn run_arg(&self, value: &str) -> String {
        format!("{}={}", self.key(), value)
    }

    /// For `podman ps --format '{{index .Labels "<key>"}}'`
    pub fn format_expr(&self) -> String {
        format!("{{{{index .Labels \"{}\"}}}}", self.key())
    }

    /// Parse a raw label value from `podman ps` output.
    /// Returns `None` for empty, whitespace-only, or the `<no value>` sentinel.
    pub fn parse_value(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "<no value>" {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_returns_oci_label_key() {
        assert_eq!(Label::Id.key(), "org.wildfly.wado.id");
        assert_eq!(Label::Topology.key(), "org.wildfly.wado.topology");
        assert_eq!(Label::Config.key(), "org.wildfly.wado.config");
    }

    #[test]
    fn filter_without_value() {
        assert_eq!(Label::Id.filter(), "label=org.wildfly.wado.id");
    }

    #[test]
    fn filter_with_value() {
        assert_eq!(
            Label::Topology.filter_value("my-topo"),
            "label=org.wildfly.wado.topology=my-topo"
        );
    }

    #[test]
    fn run_arg_formats_key_equals_value() {
        assert_eq!(
            Label::Config.run_arg("domain.xml"),
            "org.wildfly.wado.config=domain.xml"
        );
    }

    #[test]
    fn format_expr_produces_go_template() {
        assert_eq!(
            Label::Id.format_expr(),
            "{{index .Labels \"org.wildfly.wado.id\"}}"
        );
    }

    #[test]
    fn parse_value_returns_none_for_empty() {
        assert_eq!(Label::Config.parse_value(""), None);
    }

    #[test]
    fn parse_value_returns_none_for_no_value_sentinel() {
        assert_eq!(Label::Topology.parse_value("<no value>"), None);
    }

    #[test]
    fn parse_value_returns_some_for_real_value() {
        assert_eq!(
            Label::Topology.parse_value("my-topo"),
            Some("my-topo".to_string())
        );
    }

    #[test]
    fn parse_value_trims_whitespace() {
        assert_eq!(
            Label::Config.parse_value("  domain.xml  "),
            Some("domain.xml".to_string())
        );
    }
}
