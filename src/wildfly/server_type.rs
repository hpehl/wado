//! WildFly server operating modes.

use std::str::FromStr;

/// WildFly server operating mode.
///
/// Each mode corresponds to a different container image variant and
/// determines the container naming prefix (`sa`, `dc`, `hc`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_name_standalone() {
        assert_eq!(ServerType::Standalone.short_name(), "sa");
    }

    #[test]
    fn short_name_domain_controller() {
        assert_eq!(ServerType::DomainController.short_name(), "dc");
    }

    #[test]
    fn short_name_host_controller() {
        assert_eq!(ServerType::HostController.short_name(), "hc");
    }

    #[test]
    fn from_str_valid() {
        assert_eq!(ServerType::from_str("sa"), Ok(ServerType::Standalone));
        assert_eq!(ServerType::from_str("dc"), Ok(ServerType::DomainController));
        assert_eq!(ServerType::from_str("hc"), Ok(ServerType::HostController));
    }

    #[test]
    fn from_str_invalid() {
        assert_eq!(ServerType::from_str("xx"), Err(()));
        assert_eq!(ServerType::from_str(""), Err(()));
        assert_eq!(ServerType::from_str("SA"), Err(()));
    }

    #[test]
    fn ordering() {
        assert!(ServerType::Standalone < ServerType::DomainController);
        assert!(ServerType::DomainController < ServerType::HostController);
    }

    #[test]
    fn equality() {
        assert_eq!(ServerType::Standalone, ServerType::Standalone);
        assert_ne!(ServerType::Standalone, ServerType::DomainController);
    }
}
