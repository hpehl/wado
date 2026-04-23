//! Constants for container naming, image references, and environment variables.

/// Base name used in container names and image paths.
pub static WILDFLY_ADMIN_CONTAINER: &str = "wado";
/// Container image registry and organization prefix.
pub static WILDFLY_ADMIN_CONTAINER_REPOSITORY: &str = "quay.io/wado";
/// Name of the entrypoint script copied into every image.
pub static ENTRYPOINT: &str = "wado-entrypoint.sh";
/// Display width for fully qualified image names in progress output.
pub static FQN_LENGTH: usize = "quay.io/wado/wado-xx:00.0.0.Final-jdkxx".len();

/// Environment variable for JBoss CLI bootstrap operations.
pub static BOOTSTRAP_OPERATIONS_VARIABLE: &str = "WADO_BOOTSTRAP_OPERATIONS";
/// Environment variable for the domain controller hostname (used by host controllers).
pub static DOMAIN_CONTROLLER_VARIABLE: &str = "WADO_DOMAIN_CONTROLLER";
/// Environment variable for the container's logical hostname in the domain.
pub static HOSTNAME_VARIABLE: &str = "WADO_HOSTNAME";
/// Environment variable for the management user password.
pub static PASSWORD_VARIABLE: &str = "WADO_PASSWORD";
/// Environment variable for server definitions passed to domain hosts.
pub static SERVERS_VARIABLE: &str = "WADO_SERVERS";
/// Environment variable for the management user name.
pub static USERNAME_VARIABLE: &str = "WADO_USERNAME";

/// Dockerfile `RUN` command to add a management user using build secrets.
pub static ADD_USER: &str = r#"--mount=type=secret,id=username,required=true --mount=type=secret,id=password,required=true $JBOSS_HOME/bin/add-user.sh -u $(cat /run/secrets/username) -p $(cat /run/secrets/password) --silent"#;
/// Sed expression to inject CORS allowed origins into the management interface.
pub static ALLOWED_ORIGINS: &str = r#"'/allowed-origins=".*"/! s/<http-interface\(.*\)>/<http-interface\1 allowed-origins="http:\/\/localhost:1234 http:\/\/localhost:8888 http:\/\/localhost:9090 http:\/\/hal:9090 http:\/\/hal.github.io https:\/\/hal.github.io">/'"#;

/// Sed expressions to strip authentication from the management interface.
/// Three attributes are removed, each repeated across 3 passes to handle
/// multiple config files (standalone.xml, host.xml, domain.xml).
macro_rules! sed_remove_auth {
    () => {
        concat!(
            r#"-e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/'"#,
            r#" -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/'"#,
            r#" -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/' "#,
        )
    };
}
/// Combined sed expressions to strip authentication from management interfaces across all config files.
pub static NO_AUTH: &str = concat!(sed_remove_auth!(), sed_remove_auth!(), sed_remove_auth!());
