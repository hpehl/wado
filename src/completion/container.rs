use std::collections::BTreeSet;
use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;
use futures::executor::block_on;

use crate::container::query::{container_ps, running_topology_names};
use crate::wildfly::ServerType;

use super::version::parse_prefix_token;

pub fn complete_running_names(
    server_types: Vec<ServerType>,
) -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |_current: &OsStr| {
        let instances = block_on(container_ps(server_types.clone(), None, None, false));
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
        let instances = block_on(container_ps(server_types.clone(), None, None, false));
        match instances {
            Ok(instances) => {
                let versions: BTreeSet<String> = instances
                    .iter()
                    .map(|i| i.admin_container.wildfly_container.display_version())
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
        let names = block_on(running_topology_names());
        match names {
            Ok(names) => names.into_iter().map(CompletionCandidate::new).collect(),
            Err(_) => vec![],
        }
    }
}
