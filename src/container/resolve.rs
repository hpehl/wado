//! Name and port resolution for the new container starts.
//!
//! Auto-generates unique container names and non-colliding port mappings
//! based on already-running instances of the same WildFly version.

use crate::wildfly::{Ports, ResolvedStart, ServerType, StartSpec};
use std::collections::{HashMap, HashSet};
use wildfly_container_versions::WildFlyContainer;

use super::query::container_ps;

/// Resolves a list of [`StartSpec`]s into [`ResolvedStart`]s with unique names and ports.
///
/// For each spec without a custom name, a unique name is generated based on the
/// WildFly version and the number of already-running instances of the same type.
/// Ports are offset to avoid collisions with all running instances of the same version
/// (regardless of server type).
pub async fn resolve_start_specs(
    server_type: ServerType,
    specs: Vec<StartSpec>,
) -> anyhow::Result<Vec<ResolvedStart>> {
    let has_ports = server_type != ServerType::HostController;

    let needs_query: Vec<&WildFlyContainer> = specs
        .iter()
        .filter(|s| {
            s.custom_name.is_none()
                || (has_ports && (s.custom_http.is_none() || s.custom_management.is_none()))
        })
        .map(|s| &s.admin_container.wildfly_container)
        .collect();

    let mut seen = HashSet::new();
    let unique: Vec<_> = needs_query
        .into_iter()
        .filter(|wc| seen.insert(wc.identifier))
        .collect();
    let futures: Vec<_> = unique
        .iter()
        .map(|wc| running_instance_counts(server_type, wc))
        .collect();
    let results = futures::future::join_all(futures).await;
    let mut counts: HashMap<u16, (u16, u16)> = HashMap::new();
    for (wc, result) in unique.iter().zip(results) {
        let (same_type, all_types) = result?;
        counts.insert(wc.identifier, (same_type, all_types));
    }

    Ok(resolve_specs_with_counts(has_ports, &specs, &counts))
}

fn resolve_specs_with_counts(
    has_ports: bool,
    specs: &[StartSpec],
    counts: &HashMap<u16, (u16, u16)>,
) -> Vec<ResolvedStart> {
    let mut result = Vec::new();
    let chunks = specs.chunk_by(|a, b| {
        a.admin_container.wildfly_container.identifier
            == b.admin_container.wildfly_container.identifier
    });
    for chunk in chunks {
        let wc = &chunk[0].admin_container.wildfly_container;
        let (same_type, all_types) = counts.get(&wc.identifier).copied().unwrap_or((0, 0));

        let mut auto_name_counter: u16 = 0;
        for (position, spec) in chunk.iter().enumerate() {
            let name = match &spec.custom_name {
                Some(custom) => custom.clone(),
                None => {
                    let base = spec.admin_container.container_name();
                    let index = same_type + auto_name_counter;
                    auto_name_counter += 1;
                    if index > 0 {
                        format!("{}-{}", base, index)
                    } else {
                        base
                    }
                }
            };

            let ports = if has_ports {
                let port_offset = all_types + position as u16;
                let http = spec
                    .custom_http
                    .unwrap_or_else(|| wc.http_port() + port_offset);
                let management = spec
                    .custom_management
                    .unwrap_or_else(|| wc.management_port() + port_offset);
                Some(Ports { http, management })
            } else {
                None
            };

            result.push(ResolvedStart {
                admin_container: spec.admin_container.clone(),
                name,
                ports,
            });
        }
    }
    result
}

/// Counts how many instances of a given WildFly version are currently running.
///
/// Returns `(same_type, all_types)` where `same_type` is the count of instances
/// matching the given server type, and `all_types` is the total count across
/// all server types for that version.
pub async fn running_instance_counts(
    server_type: ServerType,
    wildfly_container: &WildFlyContainer,
) -> anyhow::Result<(u16, u16)> {
    let instances = container_ps(
        vec![ServerType::Standalone, ServerType::DomainController],
        Some(std::slice::from_ref(wildfly_container)),
        None,
        false,
    )
    .await?;
    let all_types = instances.len() as u16;
    let same_type = instances
        .iter()
        .filter(|i| i.admin_container.server_type == server_type)
        .count() as u16;
    Ok((same_type, all_types))
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::{AdminContainer, Ports, StartSpec};

    fn sa_spec(version: &str) -> StartSpec {
        let wc = WildFlyContainer::version(version).unwrap();
        StartSpec {
            admin_container: AdminContainer::new(wc, ServerType::Standalone),
            custom_name: None,
            custom_http: None,
            custom_management: None,
        }
    }

    fn counts(entries: &[(u16, u16, u16)]) -> HashMap<u16, (u16, u16)> {
        entries
            .iter()
            .map(|&(id, same, all)| (id, (same, all)))
            .collect()
    }

    fn resolve(specs: &[StartSpec], count_entries: &[(u16, u16, u16)]) -> Vec<ResolvedStart> {
        resolve_specs_with_counts(true, specs, &counts(count_entries))
    }

    // ------------------------------------------------------ resolve_specs_with_counts

    #[test]
    fn no_running_single_item() {
        let specs = vec![sa_spec("39")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "wado-sa-390");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base));
    }

    #[test]
    fn no_running_multiple_same_version() {
        let specs = vec![sa_spec("39"), sa_spec("39")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "wado-sa-390");
        assert_eq!(result[1].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.clone()));
        assert_eq!(result[1].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn same_type_running_single_item() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 1, 1)]);
        assert_eq!(result[0].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn different_type_running_ports_adjusted_name_unchanged() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 0, 1)]);
        assert_eq!(result[0].name, "wado-sa-390");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
    }

    #[test]
    fn different_type_running_multiple_same_version() {
        let specs = vec![sa_spec("39"), sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        let result = resolve(&specs, &[(id, 0, 1)]);
        assert_eq!(result[0].name, "wado-sa-390");
        assert_eq!(result[1].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(1)));
        assert_eq!(result[1].ports, Some(base.with_offset(2)));
    }

    #[test]
    fn mixed_running_sa_and_dc() {
        let specs = vec![sa_spec("39")];
        let id = specs[0].admin_container.wildfly_container.identifier;
        // 1 SA running (same_type=1), 2 total (SA + DC, all_type=2)
        let result = resolve(&specs, &[(id, 1, 2)]);
        assert_eq!(result[0].name, "wado-sa-390-1");
        let base = Ports::default_ports(&specs[0].admin_container.wildfly_container);
        assert_eq!(result[0].ports, Some(base.with_offset(2)));
    }

    #[test]
    fn multiple_versions_no_running() {
        let specs = vec![sa_spec("39"), sa_spec("35")];
        let result = resolve(&specs, &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "wado-sa-390");
        assert_eq!(result[1].name, "wado-sa-350");
    }

    #[test]
    fn custom_name_no_adjustment() {
        let mut spec = sa_spec("39");
        spec.custom_name = Some("my-container".to_string());
        let result = resolve(&[spec], &[]);
        assert_eq!(result[0].name, "my-container");
    }

    #[test]
    fn custom_http_only_management_adjusted() {
        let mut spec = sa_spec("39");
        spec.custom_http = Some(9000);
        let id = spec.admin_container.wildfly_container.identifier;
        let result = resolve(&[spec.clone()], &[(id, 0, 1)]);
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(ports.http, 9000);
        assert_eq!(
            ports.management,
            spec.admin_container.wildfly_container.management_port() + 1
        );
    }

    #[test]
    fn custom_management_only_http_adjusted() {
        let mut spec = sa_spec("39");
        spec.custom_management = Some(10000);
        let id = spec.admin_container.wildfly_container.identifier;
        let result = resolve(&[spec.clone()], &[(id, 0, 1)]);
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(
            ports.http,
            spec.admin_container.wildfly_container.http_port() + 1
        );
        assert_eq!(ports.management, 10000);
    }

    #[test]
    fn all_custom_no_adjustment() {
        let mut spec = sa_spec("39");
        spec.custom_name = Some("custom".to_string());
        spec.custom_http = Some(9000);
        spec.custom_management = Some(10000);
        let result = resolve(&[spec], &[(390, 2, 3)]);
        assert_eq!(result[0].name, "custom");
        let ports = result[0].ports.as_ref().unwrap();
        assert_eq!(ports.http, 9000);
        assert_eq!(ports.management, 10000);
    }

    #[test]
    fn hc_no_ports() {
        let wc = WildFlyContainer::version("39").unwrap();
        let spec = StartSpec {
            admin_container: AdminContainer::new(wc, ServerType::HostController),
            custom_name: None,
            custom_http: None,
            custom_management: None,
        };
        let result = resolve_specs_with_counts(false, &[spec], &HashMap::new());
        assert_eq!(result[0].name, "wado-hc-390");
        assert_eq!(result[0].ports, None);
    }
}
