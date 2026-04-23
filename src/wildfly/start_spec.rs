//! Start specifications and resolved container configurations.

use super::{AdminContainer, Ports};

/// Captures what the user (or topology.yml) explicitly provided for a container start.
/// `None` fields will be auto-resolved based on running instance counts.
#[derive(Clone)]
pub struct StartSpec {
    pub admin_container: AdminContainer,
    pub custom_name: Option<String>,
    pub custom_http: Option<u16>,
    pub custom_management: Option<u16>,
}

/// Resolved container name and ports, unique among running *wado* containers.
/// Non-wado collisions are caught later by `check_name_conflicts()` in `run_instances()`.
pub struct ResolvedStart {
    pub admin_container: AdminContainer,
    pub name: String,
    pub ports: Option<Ports>,
}
