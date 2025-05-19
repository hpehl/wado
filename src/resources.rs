// ------------------------------------------------------ standalone

// language=Dockerfile
pub static STANDALONE_DOCKERFILE: &str = r#"FROM {{base-image}}

LABEL maintainer="hpehl@redhat.com"
LABEL {{label-name}}="{{label-value}}"

USER root
COPY {{entrypoint}} $JBOSS_HOME/bin/{{entrypoint}}
RUN chmod +x $JBOSS_HOME/bin/{{entrypoint}}
RUN {{{add-user}}}
RUN sed -i {{{allowed-origins}}} $JBOSS_HOME/standalone/configuration/standalone*.xml
RUN for conf in $JBOSS_HOME/standalone/configuration/standalone*.xml; do sed {{{no-auth}}} "${conf}" > "${conf%%.*}-no-auth.${conf#*.}"; done
USER jboss

EXPOSE 8080 9990
ENTRYPOINT ["/opt/jboss/wildfly/bin/{{entrypoint}}", "-b", "0.0.0.0", "-bmanagement", "0.0.0.0"]
CMD ["-c", "standalone.xml"]
"#;

// language=shell script
pub static STANDALONE_ENTRYPOINT_SH: &str = r#"#!/bin/sh

if [[ ! -z $WADO_BOOTSTRAP_OPERATIONS ]]; then
    $JBOSS_HOME/bin/standalone.sh $@ --admin-only &
    until `$JBOSS_HOME/bin/jboss-cli.sh -c ":read-attribute(name=server-state)" 2> /dev/null | grep -q running`; do
        sleep 1
    done
    echo "[== Bootstrap WildFly Standalone $WILDFLY_VERSION ==]"
    echo "[-- Execute bootstrap operation: $WADO_BOOTSTRAP_OPERATIONS --]"
    $JBOSS_HOME/bin/jboss-cli.sh -c --commands="$WADO_BOOTSTRAP_OPERATIONS"
    $JBOSS_HOME/bin/jboss-cli.sh -c ":shutdown()"
    echo "[== Bootstrap finished ==]"
fi
$JBOSS_HOME/bin/standalone.sh $@
"#;

// ------------------------------------------------------ domain controller

// language=Dockerfile
pub static DOMAIN_CONTROLLER_DOCKERFILE: &str = r#"FROM {{base-image}}

LABEL maintainer="hpehl@redhat.com"
LABEL {{label-name}}="{{label-value}}"

USER root
COPY {{entrypoint}} $JBOSS_HOME/bin/{{entrypoint}}
RUN chmod +x $JBOSS_HOME/bin/{{entrypoint}}
RUN {{{add-user}}}
RUN sed -e '/<servers>/,/<\/servers>/d' -e {{{allowed-origins}}} -i $JBOSS_HOME/domain/configuration/host*.xml
RUN for conf in $JBOSS_HOME/domain/configuration/host*.xml; do sed {{{no-auth}}} "${conf}" > "${conf%%.*}-no-auth.${conf#*.}"; done
USER jboss

EXPOSE 8080 9990
ENTRYPOINT ["/opt/jboss/wildfly/bin/{{entrypoint}}", "-b", "0.0.0.0", "-bmanagement", "0.0.0.0", "--host-config", "host-{{primary}}.xml"]
CMD ["-c", "domain.xml"]
"#;

// language=shell script
pub static DOMAIN_CONTROLLER_ENTRYPOINT_SH: &str = r#"#!/bin/sh

$JBOSS_HOME/bin/domain.sh $@ --admin-only &
until `$JBOSS_HOME/bin/jboss-cli.sh -c "/host=primary:read-attribute(name=host-state)" 2> /dev/null | grep -q running`; do
    sleep 1
done
echo "[== Bootstrap WildFly Domain Controller $WILDFLY_VERSION ==]"
echo "[-- Rename primary to $WADO_HOSTNAME --]"
$JBOSS_HOME/bin/jboss-cli.sh -c --commands="/host=primary:write-attribute(name=name,value=$WADO_HOSTNAME),/host=primary:reload(admin-only)"
until `$JBOSS_HOME/bin/jboss-cli.sh -c "/host=$WADO_HOSTNAME:read-attribute(name=host-state)" 2> /dev/null | grep -q running`; do
    sleep 1
done
if [[ ! -z $WADO_SERVERS ]]; then
    echo "[-- Add servers: $WADO_SERVERS --]"
    $JBOSS_HOME/bin/jboss-cli.sh -c --commands="$WADO_SERVERS"
fi
if [[ ! -z $WADO_BOOTSTRAP_OPERATIONS ]]; then
    echo "[-- Execute bootstrap operation: $WADO_BOOTSTRAP_OPERATIONS --]"
    $JBOSS_HOME/bin/jboss-cli.sh -c --commands="$WADO_BOOTSTRAP_OPERATIONS"
fi
$JBOSS_HOME/bin/jboss-cli.sh -c "/host=$WADO_HOSTNAME:shutdown()"
echo "[== Bootstrap finished ==]"
$JBOSS_HOME/bin/domain.sh $@
"#;

// ------------------------------------------------------ host controller

// language=Dockerfile
pub static HOST_CONTROLLER_DOCKERFILE: &str = r#"FROM {{base-image}}

LABEL maintainer="hpehl@redhat.com"
LABEL {{label-name}}="{{label-value}}"

USER root
COPY {{entrypoint}} $JBOSS_HOME/bin/{{entrypoint}}
RUN chmod +x $JBOSS_HOME/bin/{{entrypoint}}
RUN {{{add-user}}}
RUN sed -e '/<servers>/,/<\/servers>/d' -e {{{allowed-origins}}} -i $JBOSS_HOME/domain/configuration/host*.xml
RUN for conf in $JBOSS_HOME/domain/configuration/host*.xml; do sed {{{no-auth}}} "${conf}" > "${conf%%.*}-no-auth.${conf#*.}"; done
USER jboss

EXPOSE 8080 9990
ENTRYPOINT ["/opt/jboss/wildfly/bin/{{entrypoint}}", "-b", "0.0.0.0", "-bmanagement", "0.0.0.0", "--host-config", "host-{{secondary}}.xml"]
CMD ["-c", "domain.xml"]
"#;

// language=shell script
pub static HOST_CONTROLLER_ENTRYPOINT_SH: &str = r#"#!/bin/sh

$JBOSS_HOME/bin/domain.sh $@ --admin-only &
until `$JBOSS_HOME/bin/jboss-cli.sh -c "/host=$HOSTNAME:read-attribute(name=host-state)" 2> /dev/null | grep -q running`; do
    sleep 1
done
echo "[== Bootstrap WildFly Host Controller $WILDFLY_VERSION ==]"
echo "[-- Rename $HOSTNAME to $WADO_HOSTNAME --]"
$JBOSS_HOME/bin/jboss-cli.sh -c --commands="/host=$HOSTNAME:write-attribute(name=name,value=$WADO_HOSTNAME),/host=$HOSTNAME:reload(admin-only)"
until `$JBOSS_HOME/bin/jboss-cli.sh -c "/host=$WADO_HOSTNAME:read-attribute(name=host-state)" 2> /dev/null | grep -q running`; do
    sleep 1
done
echo "[-- Add authentication context --]"
$JBOSS_HOME/bin/jboss-cli.sh -c --commands="/host=$WADO_HOSTNAME/subsystem=elytron/authentication-configuration=wac-auth-config:add(sasl-mechanism-selector=DIGEST-MD5,authentication-name=$WADO_USERNAME,realm=ManagementRealm,credential-reference={clear-text=$WADO_PASSWORD}),/host=$WADO_HOSTNAME/subsystem=elytron/authentication-context=wac-auth-context:add(match-rules=[{match-host=$WADO_DOMAIN_CONTROLLER,authentication-configuration=wac-auth-config}]),/host=$WADO_HOSTNAME:write-attribute(name=domain-controller.remote.authentication-context,value=wac-auth-context)"
if [[ ! -z $WADO_SERVERS ]]; then
    echo "[-- Add servers: $WADO_SERVERS --]"
    $JBOSS_HOME/bin/jboss-cli.sh -c --commands="$WADO_SERVERS"
fi
if [[ ! -z $WADO_BOOTSTRAP_OPERATIONS ]]; then
    echo "[-- Execute bootstrap operation: $WADO_BOOTSTRAP_OPERATIONS --]"
    $JBOSS_HOME/bin/jboss-cli.sh -c --commands="$WADO_BOOTSTRAP_OPERATIONS"
fi
$JBOSS_HOME/bin/jboss-cli.sh -c "/host=$WADO_HOSTNAME:shutdown()"
echo "[== Bootstrap finished ==]"
$JBOSS_HOME/bin/domain.sh $@
"#;
