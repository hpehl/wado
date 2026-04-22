//! Shell completion providers for container-related CLI arguments.
//!
//! These functions return closures compatible with [`clap_complete`]'s
//! completion engine, providing tab-completion for running container names,
//! WildFly versions, and topology names.

use std::collections::BTreeSet;
use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;
use futures::executor::block_on;

use super::query::{container_ps, running_topology_names};
use crate::wildfly::ServerType;
use crate::wildfly_version::parse_prefix_token;

/// Completes with names of running containers matching the given server types.
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

/// Completes with WildFly versions of running containers matching the given server types.
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

/// Completes with names of currently running topologies.
pub fn complete_running_topologies() -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |_current: &OsStr| {
        let names = block_on(running_topology_names());
        match names {
            Ok(names) => names
                .into_iter()
                .map(CompletionCandidate::new)
                .collect(),
            Err(_) => vec![],
        }
    }
}
