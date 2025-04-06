# WildFly Admin Containers

`waco` (**W**ildFly **a**dmin **co**ntainers) is a command line tool to build and run WildFly containers of different
versions in different operation modes (domain and standalone). The container images are based on the official WildFly
images but are intended more for the development and testing of WildFly and its management tools (CLI and console).

## Table of Contents

- [Versions](#versions)
- [Images](#images)
- [Containers](#containers)
- [Commands](#commands)
    - [Build](#build)
    - Standalone
      - [Start](#standalone-start)
      - [Stop](#standalone-stop)
    - Domain
      - Domain Controller
        - [Start](#domain-controller-start)
        - [Stop](#domain-controller-stop)
      - Host Controller
        - [Host Controller Start](#host-controller-start)
        - [Host Controller Stop](#host-controller-stop)
      - Topology
        - [Start](#topology-start)
        - [Stop](#topology-stop)
    - [PS](#ps)
    - Management Clients
      - [Console](#management-console)
      - [CLI](#cli)

## Versions

Most commands require a
WildFly [version expression](https://crates.io/crates/wildfly_container_versions#version-expressions). This could be a
single version, multiplier, range, enumeration, or a combination of all.

- 10
- 26.1
- 3x35
- 23..33
- 25..
- ..26.1
- ..
- 5x33..35
- 20,25..29,2x31,3x32,4x33..35

All supported versions are listed [here](https://crates.io/crates/wildfly_container_versions#supported-versions).

## Images

The images are based on the official WildFly images, are hosted at https://quay.io/organization/waco, and come in three
variants:

- Standalone: [quay.io/waco/waco-sa](https://quay.io/repository/waco/waco-sa)
- Domain controller: [quay.io/waco/waco-dc](https://quay.io/repository/waco/waco-dc)
- Host controller: [quay.io/waco/waco-hc](https://quay.io/repository/waco/waco-hc)

Each image contains tags for
all [supported versions](https://crates.io/crates/wildfly_container_versions#supported-versions).

### Image Modifications

The images are based on the default configuration (subsystems, profiles, server groups, socket bindings et al.) of the
corresponding version. All images add a management user `admin:admin`
and [allowed origins](https://docs.wildfly.org/34/wildscribe/core-service/management/management-interface/http-interface/index.html#attr-allowed-origins)
for

- http://localhost:1234 (used by console dev mode)
- http://localhost:8888 (used by console dev mode)
- http://localhost:9090 (used by HAL standalone)
- http://hal.github.io (latest online console)
- https://hal.github.io (latest online console)

Domain and host controller images are changed so that no servers are configured.

## Containers

The default name for containers is `waco-<version>-<type>[-index]`

- Version: `<major><minor>`
- Type: `sa|dc|hc` - standalone, domain or host controller
- Index: If multiple containers of the same version and type are used, a zero-based index is added to the name.

### Port Mappings

If not specified otherwise, the standalone and domain controller containers publish their HTTP and management ports
based on the WildFly version:

- 8080 → 8`<major><minor>`
- 9900 → 9`<major><minor>`

So for WildFly 34, the port mappings are 8340 and 9340, and for WildFly 26.1, the port mappings are 8261 and 9261.
If multiple containers of the same version are used, the port is increased by one.

## Commands

Currently, the following commands are supported:

```shell
Command line tool to build and run WildFly containers in different versions and operation modes.

Usage: waco <COMMAND>

Commands:
  build     Build WildFly images
  start     Start a standalone server
  stop      Stop a standalone server
  dc        Start and stop a domain controller
  hc        Start and stop a host controller
  topology  Start and stop a topology defined as YAML
  ps        List running images
  console   Open the management console
  cli       Connect to the CLI
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Build

```shell
Build WildFly images

Usage: waco build [OPTIONS] <wildfly-version>

Arguments:
  <wildfly-version>  A single WildFly version or version range

Options:
  -u, --username <username>  The username of the management user [default: admin]
  -p, --password <password>  The password of the management user [default: admin]
      --standalone           Build standalone images only
      --domain               Build domain controller and host controller images only
      --chunks <chunks>      Build the images in chunks of this size. If not specified, the images are built in one go.
  -h, --help                 Print help
  -V, --version              Print version
```

**Examples**

```shell
waco build 34
waco build 34 --username alice --password "Admin#70365"
waco build 10,23,34
waco build 20..29
waco build 10,20..29,34
waco build .. --chuncks 5
```

### Standalone Start

```shell
Start a standalone server

Usage: waco start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the standalone server

Options:
  -n, --name <name>              The name of the standalone server [default: waco-sa-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -p, --http <http>              The published HTTP port [default: 8<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port [default: 9<major><minor>].
                                 Not allowed when multiple versions are specified.
  -o, --offset <offset>          The offset added to the published HTTP and management ports.
                                 Not allowed when multiple versions are specified.
      --operations <operations>  A comma seperated list of operations to bootstrap the standalone server.
                                 Can be provided multiple times.
      --cli <cli>                A file with operations to bootstrap the standalone server
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
waco start 34
waco start 3x34
waco start 30..35
waco start 34 --name foo
waco start 34 --name bar --offset 100
waco start 34 --http 8080 --management 9990
waco start 34 --operations "/subsystem=logging/console-handler=CONSOLE:write-attribute(name=level,value=DEBUG)"
waco start 34 --offset 100 -- --server-config=standalone-microprofile.xml
```

### Standalone Stop

```shell
Stop a standalone server

Usage: waco stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the standalone server [default: waco-sa-<major><minor>]
  -a, --all          Stop all running standalone servers. If specified with a version,
                     stop all running standalone servers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

```shell
waco stop 34
waco stop 30..35
waco stop 34 --name foo
waco stop 34 --all
waco stop --all
```

### Domain Controller Start

```shell
Start a domain controller

Usage: waco dc start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the domain controller

Options:
  -n, --name <name>              The name of the domain controller [default: waco-dc-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -p, --http <http>              The published HTTP port [default: 8<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port [default: 9<major><minor>].
                                 Not allowed when multiple versions are specified.
  -o, --offset <offset>          The offset added to the published HTTP and management ports.
                                 Not allowed when multiple versions are specified.
  -s, --server <server>          Manage servers of the domain controller.
                                 Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].
                                 The option can be specified multiple times.
                                 <name>          The name of the server. This part is mandatory and must be first.
                                                 All other parts are optional.
                                 <server-group>  The name of the server group. Allowed values are 'main-server-group' or 'msg',
                                                 and 'other-server-group' or 'osg'. If not specified, 'main-server-group' is used.
                                 <offset>        The port offset. If not specified, 100 is used from the second server onwards.
                                 start           Whether to start the server.
      --operations <operations>  A comma seperated list of operations to bootstrap the domain controller.
                                 Can be provided multiple times.
      --cli <cli>                A file with operations to bootstrap the domain controller
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
waco dc start 34
waco dc start 3x34
waco dc start 30..35
waco dc start 34 --name foo
waco dc start 34 --name bar --offset 100
waco dc start 34 --http 8080 --management 9990
waco dc start 34 --server s1
waco dc start 35 --server s1,s2,s3,s4:osg,s5:osg
waco dc start 34 --server s1,s2,s3 --server s4:osg,s5:osg,s6:osg
waco dc start 34 --name dc \
  --server server-one:main-server-group:start \
  --server server-two:main-server-group:10 \
  --server server-three:other-server-group:20
```

### Domain Controller Stop

```shell
Stop a domain controller

Usage: waco dc stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the domain controller [default: waco-dc-<major><minor>]
  -a, --all          Stop all running domain controllers. If specified with a version,
                     stop all running domain controllers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

```shell
waco dc stop 34
waco dc stop 30..35
waco dc stop 34 --name foo
waco dc stop 34 --all
waco dc stop --all
```

### Host Controller Start

```shell
Start a host controller

Usage: waco hc start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the domain controller

Options:
  -n, --name <name>
          The name of the host controller [default: waco-hc-<major><minor>].
          Not allowed when multiple versions are specified.
  -d, --domain-controller <domain-controller>
          The name of the domain controller [default: waco-dc-<major><minor>].
          Required if different versions are specified.
  -u, --username <username>
          The username to connect to the domain controller [default: admin]
  -p, --password <password>
          The password to connect to the domain controller [default: admin]
  -s, --server <server>
          Manage servers of the host controller.
          Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].
          The option can be specified multiple times.
          <name>          The name of the server. This part is mandatory and must be first.
                          All other parts are optional.
          <server-group>  The name of the server group. Allowed values are 'main-server-group' or 'msg',
                          and 'other-server-group' or 'osg'. If not specified, 'main-server-group' is used.
          <offset>        The port offset. If not specified, 100 is used from the second server onwards.
          start           Whether to start the server.
      --operations <operations>
          A comma seperated list of operations to bootstrap the host controller.
          Can be provided multiple times.
      --cli <cli>
          A file with operations to bootstrap the host controller
  -h, --help
          Print help
  -V, --version
          Print version
```

**Examples**

```shell
waco hc start 34
waco hc start 3x34
waco hc start 30..35 --domain-controller dc
waco hc start 34 --name foo --username alice --password "Admin#70365"
waco hc start 34 --server s1
waco hc start 3x34 --server s1,s2,s3:osg
waco hc start 35 --name hc \
  --server server-one:main-server-group:start \
  --server server-two:main-server-group:10 \
  --server server-three:other-server-group:20
```

### Host Controller Stop

```shell
Stop a host controller

Usage: waco hc stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the host controller [default: waco-hc-<major><minor>]
  -a, --all          Stop all running host controllers. If specified with a version,
                     stop all running host controllers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

```shell
waco hc stop 34
waco hc stop 30..35
waco hc stop 34 --name foo
waco hc stop 34 --all
waco hc stop --all
```

### Topology Start

> **Warning**
> The topology commands are not yet implemented.

You can work around with the `dc` and `hc` commands though:

```shell
waco dc start 35 -n dc -s s1,s2,s3,s4:osg,s5:osg
waco hc start 32,33,2x35 -d dc -s s1,s2,s3:osg
waco console 35
```

Open http://localhost:9350/console/index.html#runtime;path=domain-browse-by~topology

```shell
Start a topology

Usage: waco topology start <setup>

Arguments:
  <setup>  The topology setup

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Topology Stop

```shell
Stop a topology

Usage: waco topology stop <setup>

Arguments:
  <setup>  The topology setup

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The topology setup is a YAML file like this:

```yaml
version: 34
hosts:
  - name: dc
    domain-controller: true
  - name: host1
    servers:
      - name: server-one
        group: main-server-group
        auto-start: true
      - name: server-two
        group: main-server-group
        offset: 100
      - name: server-three
        group: other-server-group
        offset: 200
      - name: server-four
        group: other-server-group
        offset: 300
  - name: host2
    version: 33
    servers:
      - name: server-one
        group: main-server-group
      - name: server-two
        group: main-server-group
        offset: 100
      - name: server-three
        group: other-server-group
        offset: 200
  - name: host3
    servers:
      - name: server-one
        group: main-server-group
      - name: server-two
        group: other-server-group
        offset: 100
      - name: server-three
        group: other-server-group
        offset: 200
```

### PS

```shell
List running standalone, domain and host controller containers

Usage: waco ps

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Management Console

```shell
Open the management console

Usage: waco console [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>              The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port. Not allowed when multiple versions are specified.
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
waco console 34
waco console 30..35
waco console 34 --management 9990
```

### CLI

```shell
Connect to the CLI

Usage: waco cli [OPTIONS] [wildfly-version] [-- [cli-parameters]...]

Arguments:
  [wildfly-version]    A single WildFly version
  [cli-parameters]...  Parameters passed to the CLI

Options:
  -n, --name <name>              The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port
  -u, --username <username>      The username to connect to the CLI [default: admin]
  -p, --password <password>      The password to connect to the CLI [default: admin]
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
waco cli 34
waco cli 34 -- --command "/subsystem=logging/console-handler=CONSOLE:write-attribute(name=level,value=DEBUG)"
```
