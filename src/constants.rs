pub static WILDFLY_ADMIN_CONTAINER: &str = "wfadm";
pub static WILDFLY_ADMIN_CONTAINER_REPOSITORY: &str = "quay.io/wfadm";
pub static ENTRYPOINT: &str = "wfadm-entrypoint.sh";
pub static LABEL_NAME: &str = "org.wildfly.wfadm.id";

pub static BOOTSTRAP_OPERATIONS_VARIABLE: &str = "WFADM_BOOTSTRAP_OPERATIONS";
pub static DOMAIN_CONTROLLER_VARIABLE: &str = "WFADM_DOMAIN_CONTROLLER";
pub static HOSTNAME_VARIABLE: &str = "WFADM_HOSTNAME";
pub static PASSWORD_VARIABLE: &str = "WFADM_PASSWORD";
pub static SERVERS_VARIABLE: &str = "WFADM_SERVERS";
pub static USERNAME_VARIABLE: &str = "WFADM_USERNAME";

pub static ADD_USER: &str = r#"--mount=type=secret,id=username,required=true --mount=type=secret,id=password,required=true $JBOSS_HOME/bin/add-user.sh -u $(cat /run/secrets/username) -p $(cat /run/secrets/password) --silent"#;
pub static ALLOWED_ORIGINS: &str = r#"'/allowed-origins=".*"/! s/<http-interface\(.*\)>/<http-interface\1 allowed-origins="http:\/\/localhost:1234 http:\/\/localhost:8888 http:\/\/localhost:9090 http:\/\/hal:9090 http:\/\/hal.github.io https:\/\/hal.github.io">/'"#;
pub static NO_AUTH: &str = r#"-e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/' -e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/' -e 's/<http-interface\(.*\)security-realm="ManagementRealm"\(.*\)>/<http-interface\1\2>/' -e 's/<http-interface\(.*\)http-authentication-factory="management-http-authentication"\(.*\)>/<http-interface\1\2>/' -e 's/<http-upgrade\(.*\)sasl-authentication-factory="management-sasl-authentication"\(.*\)\/>/<http-upgrade\1\2\/>/'"#;
