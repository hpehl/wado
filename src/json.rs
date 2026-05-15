use crate::error::WadoErrorCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct CommandResult {
    pub identifier: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<WadoErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CommandResult {
    pub fn success(identifier: &str, http: Option<u16>, management: Option<u16>) -> Self {
        Self {
            identifier: identifier.to_string(),
            success: true,
            http,
            management,
            error_code: None,
            error: None,
        }
    }

    pub fn error(identifier: &str, error: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            success: false,
            http: None,
            management: None,
            error_code: Some(WadoErrorCode::ContainerCommandFailed),
            error: Some(error.to_string()),
        }
    }
}

#[derive(Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub server_type: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology: Option<String>,
    pub status: String,
    pub container_id: String,
}

#[derive(Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub wildfly_version: String,
    pub core_version: String,
    pub repository: String,
}
