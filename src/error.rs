use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WadoErrorCode {
    ContainerRuntimeNotFound,
    ContainerCommandFailed,
    ContainerStartFailed,
    ContainerStopFailed,
    ContainerListFailed,
    ImageListFailed,
    RegistryInitFailed,
    UnknownVersion,
    TopologyError,
    ClapParseError,
    Internal,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
#[must_use]
pub struct WadoError {
    pub code: WadoErrorCode,
    pub message: String,
}

#[allow(dead_code)]
impl WadoError {
    pub fn container_runtime_not_found() -> Self {
        Self {
            code: WadoErrorCode::ContainerRuntimeNotFound,
            message: "Neither podman nor docker found. Install one of them to continue".into(),
        }
    }

    pub fn container_command_failed(context: &str, stderr: &str) -> Self {
        Self {
            code: WadoErrorCode::ContainerCommandFailed,
            message: format!("{context}: {stderr}"),
        }
    }

    pub fn container_start_failed(name: &str, stderr: &str) -> Self {
        Self {
            code: WadoErrorCode::ContainerStartFailed,
            message: format!("Failed to start container {name}: {stderr}"),
        }
    }

    pub fn container_stop_failed(name: &str, stderr: &str) -> Self {
        Self {
            code: WadoErrorCode::ContainerStopFailed,
            message: format!("Failed to stop container {name}: {stderr}"),
        }
    }

    pub fn container_list_failed(stderr: &str) -> Self {
        Self {
            code: WadoErrorCode::ContainerListFailed,
            message: format!("Failed to list containers: {stderr}"),
        }
    }

    pub fn image_list_failed(stderr: &str) -> Self {
        Self {
            code: WadoErrorCode::ImageListFailed,
            message: format!("Failed to list images: {stderr}"),
        }
    }

    pub fn registry_init_failed(details: &str) -> Self {
        Self {
            code: WadoErrorCode::RegistryInitFailed,
            message: format!("Failed to initialize registry: {details}"),
        }
    }

    pub fn unknown_version(input: &str) -> Self {
        Self {
            code: WadoErrorCode::UnknownVersion,
            message: format!(
                "\"{input}\" is not a known WildFly version. \
                 Use 'wado versions' to list available versions."
            ),
        }
    }

    pub fn topology_error(details: &str) -> Self {
        Self {
            code: WadoErrorCode::TopologyError,
            message: details.to_string(),
        }
    }

    pub fn clap_parse_error(details: &str) -> Self {
        Self {
            code: WadoErrorCode::ClapParseError,
            message: details.to_string(),
        }
    }

    pub fn error_code(err: &anyhow::Error) -> WadoErrorCode {
        err.downcast_ref::<WadoError>()
            .map(|e| e.code)
            .unwrap_or(WadoErrorCode::Internal)
    }
}

#[derive(Serialize)]
pub struct JsonErrorEnvelope {
    pub error: JsonErrorBody,
}

#[derive(Serialize)]
pub struct JsonErrorBody {
    pub code: WadoErrorCode,
    pub message: String,
}

impl JsonErrorEnvelope {
    pub fn from_anyhow(err: &anyhow::Error) -> Self {
        match err.downcast_ref::<WadoError>() {
            Some(wado) => Self {
                error: JsonErrorBody {
                    code: wado.code,
                    message: wado.message.clone(),
                },
            },
            None => Self {
                error: JsonErrorBody {
                    code: WadoErrorCode::Internal,
                    message: err.to_string(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serializes_as_screaming_snake_case() {
        let json = serde_json::to_string(&WadoErrorCode::ContainerRuntimeNotFound).unwrap();
        assert_eq!(json, "\"CONTAINER_RUNTIME_NOT_FOUND\"");
    }

    #[test]
    fn error_code_unknown_version() {
        let json = serde_json::to_string(&WadoErrorCode::UnknownVersion).unwrap();
        assert_eq!(json, "\"UNKNOWN_VERSION\"");
    }

    #[test]
    fn wado_error_display_uses_message() {
        let err = WadoError::container_runtime_not_found();
        assert_eq!(
            err.to_string(),
            "Neither podman nor docker found. Install one of them to continue"
        );
    }

    #[test]
    fn wado_error_parameterized_message() {
        let err = WadoError::container_start_failed("wado-sa-360", "port already in use");
        assert_eq!(
            err.to_string(),
            "Failed to start container wado-sa-360: port already in use"
        );
    }

    #[test]
    fn json_error_envelope_from_anyhow_with_wado_error() {
        let err: anyhow::Error = WadoError::registry_init_failed("network timeout").into();
        let envelope = JsonErrorEnvelope::from_anyhow(&err);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(json["error"]["code"], "REGISTRY_INIT_FAILED");
        assert!(
            json["error"]["message"]
                .as_str()
                .unwrap()
                .contains("network timeout")
        );
    }

    #[test]
    fn json_error_envelope_from_anyhow_without_wado_error() {
        let err = anyhow::anyhow!("something unexpected");
        let envelope = JsonErrorEnvelope::from_anyhow(&err);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(json["error"]["code"], "INTERNAL");
        assert_eq!(json["error"]["message"], "something unexpected");
    }

    #[test]
    fn error_code_extracts_from_anyhow() {
        let err: anyhow::Error = WadoError::topology_error("missing dc").into();
        assert_eq!(WadoError::error_code(&err), WadoErrorCode::TopologyError);
    }

    #[test]
    fn error_code_falls_back_to_internal() {
        let err = anyhow::anyhow!("plain error");
        assert_eq!(WadoError::error_code(&err), WadoErrorCode::Internal);
    }
}
