![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/hpehl/wado/verify.yml)
[![Crates.io](https://img.shields.io/crates/v/wado.svg)](https://crates.io/crates/wado)

# WildFly Admin Containers

`wado` (**W**ildFly **ad**min c**o**ntainers) is a command line tool to build and run WildFly containers of different
versions in different operation modes (domain and standalone). The container images are based on the official WildFly
images but are intended more for the development and testing of WildFly and its management tools (CLI and console).
The container names and published ports follow default values based on the WildFly version.

- [Installation](#installation)
- [Versions](#versions)
- [Images](#images)
- [Containers](#containers)
- [Commands](#commands)
    - [Build](#build)
        - [Dev Build](#dev-build)
    - [Standalone](#standalone)
        - [Start](#start)
        - [Stop](#stop)
    - [Domain](#domain)
        - [Domain Controller](#domain-controller)
            - [Start](#start-1)
            - [Stop](#stop-1)
        - [Host Controller](#host-controller)
            - [Start](#start-2)
            - [Stop](#stop-2)
        - [Topology](#topology)
    - [Images](#images-1)
    - [PS](#ps)
    - [Management Clients](#management-clients)
        - [Console](#console)
        - [CLI](#cli)

# Installation

[Precompiled binaries](https://github.com/hpehl/wado/releases) are available for macOS, Linux, and Windows.

## Brew

```shell
brew tap hpehl/tap
brew install wado
```

## Cargo

```shell
cargo install wado
```

## Build from source

1. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (
   see [Install Rust and Cargo](https://www.rust-lang.org/tools/install))
2. `git clone git@github.com:hpehl/wado.git`
3. `cd wado`
4. `cargo build --release && cargo install --path .`

This installs the `wado` binary to `~/.cargo/bin/` which should be in you `$PATH`.

## Shell Completions

`wado` provides dynamic shell completions including WildFly version suggestions. The easiest way to set them up is:

```shell
wado completions --install
```

This auto-detects your shell and installs the completion script to the standard location. You can also specify the shell explicitly:

```shell
wado completions fish --install
```

To print the completion script to stdout (e.g. for manual setup or piping):

```shell
wado completions fish
```

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

# Versions

Most commands require a WildFly version expression.
Version expressions are either short versions, multipliers, ranges, enumerations, or a combination of them.
They follow
this [BNF](https://bnfplayground.pauliankline.com/?bnf=%3Cexpression%3E%20%3A%3A%3D%20%3Cexpression%3E%20%22%2C%22%20%3Celement%3E%20%7C%20%3Celement%3E%0A%3Celement%3E%20%3A%3A%3D%20%3Cmultiplier%3E%20%22x%22%20%3Crange%3E%20%7C%20%3Cmultiplier%3E%20%22x%22%20%3Cshort_version%3E%20%7C%20%3Crange%3E%20%7C%20%3Cshort_version%3E%0A%3Crange%3E%20%3A%3A%3D%20%3Cshort_version%3E%20%22..%22%20%3Cshort_version%3E%20%7C%20%22..%22%20%3Cshort_version%3E%20%7C%20%3Cshort_version%3E%20%22..%22%20%7C%20%22..%22%0A%3Cmultiplier%3E%20%3A%3A%3D%20%3Cnonzero_number%3E%20%7C%20%3Ctwo_digit_number%3E%0A%3Cshort_version%3E%20%3A%3A%3D%20%3Cmajor%3E%20%7C%20%3Cmajor%3E%20%22.%22%20%3Cminor%3E%0A%3Cmajor%3E%20%3A%3A%3D%20%3Ctwo_digit_number%3E%20%7C%20%3Cthree_digit_number%3E%0A%3Cminor%3E%20%3A%3A%3D%20%3Cnonzero_number%3E%20%7C%20%3Ctwo_digit_number%3E%0A%3Cthree_digit_number%3E%20%3A%3A%3D%20%3Cnonzero_number%3E%20%3Cnumber%3E%20%3Cnumber%3E%0A%3Ctwo_digit_number%3E%20%3A%3A%3D%20%3Cnonzero_number%3E%20%3Cnumber%3E%0A%3Cnumber%3E%20%3A%3A%3D%20%220%22%20%7C%20%221%22%20%7C%20%222%22%20%7C%20%223%22%20%7C%20%224%22%20%7C%20%225%22%20%7C%20%226%22%20%7C%20%227%22%20%7C%20%228%22%20%7C%20%229%22%0A%3Cnonzero_number%3E%20%3A%3A%3D%20%221%22%20%7C%20%222%22%20%7C%20%223%22%20%7C%20%224%22%20%7C%20%225%22%20%7C%20%226%22%20%7C%20%227%22%20%7C%20%228%22%20%7C%20%229%22%0A&name=WildFly%20Container%20Versions):

```
<expression> ::= <expression> "," <element> | <element>
<element> ::= <multiplier> "x" <range> | <multiplier> "x" <short_version> | <range> | <short_version>
<range> ::= <short_version> ".." <short_version> | ".." <short_version> | <short_version> ".." | ".."
<multiplier> ::= <nonzero_number> | <two_digit_number>
<short_version> ::= <major> | <major> "." <minor>
<major> ::= <two_digit_number> | <three_digit_number>
<minor> ::= <nonzero_number> | <two_digit_number>
<three_digit_number> ::= <nonzero_number> <number> <number>
<two_digit_number> ::= <nonzero_number> <number>
<number> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
<nonzero_number> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
```

**Examples**

- 10
- 26.1
- 3x35
- 23..33
- 25..
- ..26.1
- ..
- 5x33..35
- 20,25..29,2x31,3x32,4x33..35

All supported versions are
listed [here](https://github.com/hpehl/wildfly-container-versions?tab=readme-ov-file#supported-versions).

## Dev Version

In addition to released versions, the special keyword `dev` can be used to build WildFly from source. Dev builds clone
and compile the [WildFly](https://github.com/wildfly/wildfly) and [HAL console](https://github.com/hal/console)
repositories from GitHub, integrate the console into the WildFly distribution, and build container images from the
result. See [Build](#build) for details.

Dev containers use the name `wado-<type>-dev` (e.g., `wado-sa-dev`) and the ports `8000` / `9000`.

> [!NOTE]
> Dev builds cannot be mixed with versioned builds. Use `wado build dev` or `wado build <versions>`, but not both.

# Images

The images are based on the official WildFly images, are hosted at https://quay.io/organization/wado, and come in three
variants:

- Standalone: [quay.io/wado/wado-sa](https://quay.io/repository/wado/wado-sa)
- Domain controller: [quay.io/wado/wado-dc](https://quay.io/repository/wado/wado-dc)
- Host controller: [quay.io/wado/wado-hc](https://quay.io/repository/wado/wado-hc)

Each image contains tags for
all [supported versions](https://github.com/hpehl/wildfly-container-versions?tab=readme-ov-file#supported-versions).

## Image Modifications

The images are based on the default configuration (subsystems, profiles, server groups, socket bindings et al.) of the
corresponding version. Unless specified otherwise, the images use these configuration files to start WildFly:

- Standalone: `standalone.xml`
- Domain controller: `domain.xml` and `host-primary.xml`
- Host controller: `domain.xml` and `host-secondary.xml`

All images add a management user `admin:admin`
and [allowed origins](https://docs.wildfly.org/36/wildscribe/core-service/management/management-interface/http-interface/index.html#attr-allowed-origins)
for

- http://localhost:1234 (used by console dev mode)
- http://localhost:8888 (used by console dev mode)
- http://localhost:9090 (used by HAL standalone)
- http://hal.github.io (latest online console)
- https://hal.github.io (latest online console)

Domain and host controller images are changed so that no servers are configured.

# Containers

## Naming

The default name for containers is `wado-<type>-<version>[-index]`

- Type: `sa|dc|hc` - standalone, domain or host controller
- Version: `<major><minor>` or `dev` for dev builds
- Index: If multiple containers of the same version and type are used, a zero-based index is added to the name.

## Ports

If not specified otherwise, the standalone and domain controller containers publish their HTTP and management ports
based on the WildFly version:

- 8080 → 8`<major><minor>`
- 9900 → 9`<major><minor>`

So for WildFly 34, the port mappings are 8340 and 9340, and for WildFly 26.1, the port mappings are 8261 and 9261.
If multiple containers of the same version are used, the port is increased by one from the second container onwards.

```shell
wado start 26.1,28..30,2x32,3x35,dev
```

| Version | Name          | HTTP | Management |
|---------|---------------|------|------------|
| 26.1    | wado-sa-261   | 8261 | 9261       |
| 28      | wado-sa-280   | 8280 | 9280       |
| 29      | wado-sa-290   | 8290 | 9290       |
| 30      | wado-sa-300   | 8300 | 9300       |
| 32      | wado-sa-320-0 | 8320 | 9320       |
| 32      | wado-sa-320-1 | 8321 | 9321       |
| 35      | wado-sa-350-0 | 8350 | 9350       |
| 35      | wado-sa-350-1 | 8351 | 9351       |
| 35      | wado-sa-350-2 | 8352 | 9352       |
| dev     | wado-sa-dev   | 8000 | 9000       |

# Commands

> [!IMPORTANT]
> Most commands require `podman` to be present with `docker` as a fallback.
> The `console` command opens the default web browser and the `cli` command requires a JVM.

## Build

Builds standalone, domain controller, and host controller images based on the official WildFly images. The images contain specific [modifications](#image-modifications) and a management user (
`admin:admin` by default). You can restrict the build to `--standalone` or `--domain` images only, and use
`--chunks` to build in parallel batches.

Predefined images for all [supported versions](https://github.com/hpehl/wildfly-container-versions?tab=readme-ov-file#supported-versions) are available at https://quay.io/organization/wado. Build your own local images if you want to change the username and password.

```shell
wado build 34
wado build 34 --username alice --password "Admin#70365"
wado build 10,23,34 --standalone
wado build 20..29 --domain
wado build 10,20..29,34
wado build .. --chunks 5
```

### Dev Build

Use
`dev` as the version to build WildFly from source. This clones and compiles the [WildFly](https://github.com/wildfly/wildfly) and [HAL console](https://github.com/hal/console) repositories, integrates the console into the distribution, and builds container images from the result. Use
`--wildfly-branch` and `--hal-branch` to control which branches to build from (both default to `main`), and
`--verbose` to see the full Maven build output.

```shell
wado build dev
wado build dev --standalone
wado build dev --wildfly-branch my-feature-branch
wado build dev --hal-branch my-console-branch
wado build dev --verbose
```

## Standalone

### Start

Starts one or more standalone WildFly containers. Container names and ports are derived from the version by default (see [Containers](#containers)). You can override the name, HTTP port, management port, or apply a port offset for single-version starts. Use
`--operations` or `--cli` to bootstrap the server with management operations. Additional WildFly parameters can be passed after
`--`.

```shell
wado start 34
wado start 3x34
wado start 30..35
wado start 34 --name foo
wado start 34 --name bar --offset 100
wado start 34 --http 8080 --management 9990
wado start 34 --operations "/subsystem=logging/console-handler=CONSOLE:write-attribute(name=level,value=DEBUG)"
wado start 34 --offset 100 -- --server-config=standalone-microprofile.xml
```

### Stop

Stops standalone containers by version, name, or all at once.

```shell
wado stop 34
wado stop 30..35
wado stop 34 --name foo
wado stop 34 --all
wado stop --all
```

## Domain

### Domain Controller

#### Start

Starts one or more domain controllers. Supports the same naming, port, and offset options as standalone. Use
`--server` to configure servers on the domain controller. Servers are specified as
`<name>[:<server-group>][:<offset>][:start]`, where the server group defaults to `main-server-group` (shorthand `msg`) and
`other-server-group` can be abbreviated as `osg`. If no offset is specified, it is auto-incremented by 100 from the second server onward (0, 100, 200, ...).

```shell
wado dc start 34
wado dc start 3x34
wado dc start 30..35
wado dc start 34 --name foo
wado dc start 34 --name bar --offset 100
wado dc start 34 --http 8080 --management 9990
wado dc start 34 --server s1:start
wado dc start 35 --server s1,s2,s3,s4:osg,s5:osg
wado dc start 34 --server s1:start,s2,s3 --server s4:osg:start,s5:osg,s6:osg
wado dc start 34 --name dc \
  --server server-one:main-server-group:start \
  --server server-two:main-server-group:10 \
  --server server-three:other-server-group:20
```

#### Stop

Stops domain controllers by version, name, or all at once.

```shell
wado dc stop 34
wado dc stop 30..35
wado dc stop 34 --name foo
wado dc stop 34 --all
wado dc stop --all
```

### Host Controller

#### Start

Starts one or more host controllers that connect to a running domain controller. The domain controller defaults to
`wado-dc-<major><minor>` but can be specified with
`--domain-controller`. That means a running domain controller of the same WildFly version will be found automatically. Use
`--server` to configure servers (same syntax as the domain controller). Credentials for connecting to the domain controller default to
`admin:admin`.

```shell
wado hc start 34
wado hc start 3x34
wado hc start 30..35 --domain-controller dc
wado hc start 34 -n foo -d dc -u alice -p "Admin#70365"
wado hc start 34 --server s1
wado hc start 3x34 --server s1,s2,s3:osg
wado hc start 35 --name hc \
  --server server-one:main-server-group:start \
  --server server-two:main-server-group:10 \
  --server server-three:other-server-group:20
```

#### Stop

Stops host controllers by version, name, or all at once.

```shell
wado hc stop 34
wado hc stop 30..35
wado hc stop 34 --name foo
wado hc stop 34 --all
wado hc stop --all
```

### Topology

Starts or stops a complete domain topology defined as a YAML file. The topology file specifies the domain controller, host controllers, their servers, and optionally mixed WildFly versions. When stopping, you can pass either the YAML file or just the topology name.

```shell
wado topology start my-topology.yaml
wado topology stop my-topology.yaml
wado topology stop my-topology
```

#### Topology File Format

The topology file is a YAML file with the following structure:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Name of the topology |
| `version` | number | yes | WildFly version used for all hosts (unless overridden per host) |
| `hosts` | list | yes | List of hosts in the topology |

Each host supports the following fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | no | `wado-dc-<major><minor>` / `wado-hc-<major><minor>` | Name of the host. Defaults to the standard container name based on the server type and version. Must be unique if provided. |
| `domain-controller` | bool | no | `false` | Whether this host is the domain controller. Exactly one host must be the domain controller. |
| `version` | number | no | top-level version | WildFly version override for this host. Allows mixed-version topologies. |
| `servers` | list | no | `[]` | List of servers on this host |

Each server supports the following fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | yes | - | Name of the server |
| `group` | string | yes | - | Server group: `main-server-group` (or `msg`) / `other-server-group` (or `osg`) |
| `offset` | number | no | `0` | Socket binding port offset. If not specified, auto-incremented by 100 from the second server onward (0, 100, 200, ...). |
| `auto-start` | bool | no | `false` | Whether to auto-start the server when the host starts |

#### Example

```yaml
name: my-topology
version: 39
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
        offset: 10
      - name: server-three
        group: other-server-group
        offset: 20
  - name: host2
    version: 38
    servers:
      - name: server-one
        group: main-server-group
      - name: server-two
        group: main-server-group
      - name: server-three
        group: other-server-group
  - servers:
      - name: server-one
        group: main-server-group
      - name: server-two
        group: other-server-group
      - name: server-three
        group: other-server-group
```

## Images

Lists all locally available standalone, domain controller, and host controller images.

```shell
wado images
```

## PS

Lists all running wado containers. Use `--standalone` or `--domain` to filter by operation mode.

```shell
wado ps
wado ps --standalone
wado ps --domain
```

## Management Clients

### Console

Opens the WildFly management console in the default web browser. If no version is specified, the console is opened for all running standalone and domain controller containers.

```shell
wado console
wado console 34
wado console 30..35
wado console 34 --management 9990
```

### CLI

Connects to the JBoss CLI of a running container. If not already present, this command downloads the
`wildfly-cli-client.jar` and `jboss-cli.xml` of the specified version to
`$TMPDIR`. The version can be omitted if only one standalone or domain controller is running. Additional CLI parameters can be passed after
`--`.

```shell
wado cli
wado cli 34
wado cli 34 -- --command "/subsystem=logging/console-handler=CONSOLE:write-attribute(name=level,value=DEBUG)"
```
