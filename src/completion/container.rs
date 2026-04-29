use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::sync::OnceLock;

use clap_complete::engine::CompletionCandidate;
use futures::executor::block_on;
use wildfly_meta::WildFlyImageRegistry;

use crate::container::query::{container_ps, running_topology_names};
use crate::wildfly::ServerType;

use super::version::parse_prefix_token;

static REGISTRY: OnceLock<WildFlyImageRegistry> = OnceLock::new();

fn registry() -> &'static WildFlyImageRegistry {
    REGISTRY.get_or_init(|| {
        WildFlyImageRegistry::load_default().expect("failed to load image registry")
    })
}

pub fn complete_running_names(
    server_types: Vec<ServerType>,
) -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |_current: &OsStr| {
        let instances = block_on(container_ps(
            server_types.clone(),
            None,
            None,
            false,
            registry(),
        ));
        match instances {
            Ok(instances) => instances
                .iter()
                .map(|i| CompletionCandidate::new(i.name.clone()))
                .collect(),
            Err(_) => vec![],
        }
    }
}

pub fn complete_running_versions(
    server_types: Vec<ServerType>,
) -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |current: &OsStr| {
        let input = current.to_str().unwrap_or("");
        let (prefix, _token) =
            parse_prefix_token(if input.is_empty() { None } else { Some(input) });
        let instances = block_on(container_ps(
            server_types.clone(),
            None,
            None,
            false,
            registry(),
        ));
        match instances {
            Ok(instances) => {
                let versions: BTreeSet<String> = instances
                    .iter()
                    .map(|i| i.admin_image.wildfly_image.short_name())
                    .collect();
                versions
                    .iter()
                    .map(|v| CompletionCandidate::new(format!("{}{}", prefix, v)))
                    .collect()
            }
            Err(_) => vec![],
        }
    }
}

pub fn complete_running_topologies() -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |_current: &OsStr| {
        let names = block_on(running_topology_names(registry()));
        match names {
            Ok(names) => names.into_iter().map(CompletionCandidate::new).collect(),
            Err(_) => vec![],
        }
    }
}
