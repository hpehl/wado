// ------------------------------------------------------ dockerfile

// Unified Dockerfile template for all server types and build modes (dev/stable).
// Conditionals:
//   is-dev — set for dev builds (adds JDK base image, ENV, user setup, COPY wildfly)
//   is-standalone — set for a standalone server type (uses standalone config paths)
//   host-config — set for DC/HC (e.g. "host-primary.xml"), controls ENTRYPOINT/CMD
//   base-image — set for stable builds (the upstream WildFly image)

// language=Dockerfile
pub static DOCKERFILE: &str = r#"{{#if is-dev~}}
FROM eclipse-temurin:21-ubi9-minimal

RUN microdnf update -y && \
    microdnf install --best --nodocs -y unzip && \
    microdnf clean all

RUN groupadd -r jboss -g 1000 && useradd -u 1000 -r -g jboss -m -d /opt/jboss -s /sbin/nologin -c "JBoss user" jboss && \
    chmod 755 /opt/jboss

ENV JBOSS_HOME=/opt/jboss/wildfly
ENV WILDFLY_VERSION=development

COPY wildfly $JBOSS_HOME
{{~else~}}
FROM {{base-image}}
{{~/if}}

LABEL maintainer="hpehl@redhat.com"
LABEL {{label-name}}="{{label-value}}"

USER root
COPY {{entrypoint}} $JBOSS_HOME/bin/{{entrypoint}}
RUN chmod +x $JBOSS_HOME/bin/{{entrypoint}}
RUN {{{add-user}}}
{{#if is-standalone~}}
RUN sed -i {{{allowed-origins}}} $JBOSS_HOME/standalone/configuration/standalone*.xml
RUN for conf in $JBOSS_HOME/standalone/configuration/standalone*.xml; do sed {{{no-auth}}} "${conf}" > "${conf%%.*}-no-auth.${conf#*.}"; done
{{else}}
RUN sed -e '/<servers>/,/<\/servers>/d' -e {{{allowed-origins}}} -i $JBOSS_HOME/domain/configuration/host*.xml
RUN for conf in $JBOSS_HOME/domain/configuration/host*.xml; do sed {{{no-auth}}} "${conf}" > "${conf%%.*}-no-auth.${conf#*.}"; done
{{/if}}
{{#if is-dev~}}
RUN chown -R jboss:0 ${JBOSS_HOME} && \
    chmod -R g+rw ${JBOSS_HOME}
ENV LAUNCH_JBOSS_IN_BACKGROUND=true
{{/if~}}
USER jboss

EXPOSE 8080 9990
{{#if host-config~}}
ENTRYPOINT ["/opt/jboss/wildfly/bin/{{entrypoint}}", "-b", "0.0.0.0", "-bmanagement", "0.0.0.0", "--host-config", "{{host-config}}"]
CMD ["-c", "domain.xml"]
{{~else~}}
ENTRYPOINT ["/opt/jboss/wildfly/bin/{{entrypoint}}", "-b", "0.0.0.0", "-bmanagement", "0.0.0.0"]
CMD ["-c", "standalone.xml"]
{{~/if}}
"#;

// ------------------------------------------------------ standalone

// language=shell script
pub static STANDALONE_ENTRYPOINT_SH: &str = r#"#!/bin/bash

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

// language=shell script
pub static DOMAIN_CONTROLLER_ENTRYPOINT_SH: &str = r#"#!/bin/bash

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

// language=shell script
pub static HOST_CONTROLLER_ENTRYPOINT_SH: &str = r#"#!/bin/bash

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
