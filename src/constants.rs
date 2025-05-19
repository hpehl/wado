pub static WILDFLY_ADMIN_CONTAINER: &str = "wado";
pub static WILDFLY_ADMIN_CONTAINER_REPOSITORY: &str = "quay.io/wado";
pub static ENTRYPOINT: &str = "wado-entrypoint.sh";
pub static LABEL_NAME: &str = "org.wildfly.wado.id";
pub static FQN_LENGTH: usize = "quay.io/wado/wado-xx:00.0.0.Final-jdkxx".len();

pub static BOOTSTRAP_OPERATIONS_VARIABLE: &str = "WADO_BOOTSTRAP_OPERATIONS";
pub static DOMAIN_CONTROLLER_VARIABLE: &str = "WADO_DOMAIN_CONTROLLER";
pub static HOSTNAME_VARIABLE: &str = "WADO_HOSTNAME";
pub static PASSWORD_VARIABLE: &str = "WADO_PASSWORD";
pub static SERVERS_VARIABLE: &str = "WADO_SERVERS";
pub static USERNAME_VARIABLE: &str = "WADO_USERNAME";

pub static ADD_USER: &str = r#"--mount=type=secret,id=username,required=true --mount=type=secret,id=password,required=true $JBOSS_HOME/bin/add-user.sh -u $(cat /run/secrets/username) -p $(cat /run/secrets/password) --silent"#;
pub static ALLOWED_ORIGINS: &str = r#"'/allowed-origins=".*"/! s/<http-interface\(.*\)>/<http-interface\1 allowed-origins="http:\/\/localhost:1234 http:\/\/localhost:8888 http:\/\/localhost:9090 http:\/\/hal:9090 http:\/\/hal.github.io https:\/\/hal.github.io">/'"#;
pub static NO_AUTH: &str = r#"-e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/' -e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/' -e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/'"#;
