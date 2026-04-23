//! WildFly server operating modes.

use std::str::FromStr;

/// WildFly server operating mode.
///
/// Each mode corresponds to a different container image variant and
/// determines the container naming prefix (`sa`, `dc`, `hc`).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServerType {
    Standalone,
    DomainController,
    HostController,
}

impl ServerType {
    /// Returns the two-letter abbreviation used in container names and identifiers.
    pub fn short_name(&self) -> &'static str {
        match self {
            ServerType::Standalone => "sa",
            ServerType::DomainController => "dc",
            ServerType::HostController => "hc",
        }
    }
}

impl FromStr for ServerType {
    type Err = ();

    fn from_str(input: &str) -> Result<ServerType, Self::Err> {
        match input {
            "sa" => Ok(ServerType::Standalone),
            "dc" => Ok(ServerType::DomainController),
            "hc" => Ok(ServerType::HostController),
            _ => Err(()),
        }
    }
}
