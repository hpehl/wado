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
            - [Start](#start-3)
            - [Stop](#stop-3)
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

<details>
<summary>The repository contains shell completions for bash, fish, zsh, elvish, and PowerShell.
They're installed automatically by brew. To install them manually, follow these steps:</summary>

### Bash

```shell
wget https://github.com/hpehl/wado/raw/main/completions/wado.bash -O /etc/bash_completion.d/wado
source /etc/bash_completion.d/wado
```

### Zsh

```shell
wget https://github.com/hpehl/wado/raw/main/completions/_wado -O /usr/local/share/zsh/site-functions/_wado
autoload -U compinit && compinit
autoload -U _wado
```

### Fish

```shell
wget https://github.com/hpehl/wado/raw/main/completions/wado.fish -O ~/.config/fish/completions/wado.fish
```

### Elvish

```shell
wget https://github.com/hpehl/wado/raw/main/completions/wado.elv -O ~/.elvish/lib/wado.elv
```

### PowerShell

```shell
Invoke-WebRequest -Uri https://github.com/hpehl/wado/raw/main/completions/_wado.ps1 -OutFile "$HOME\.config\powershell\_wado.ps1"
. "$HOME\.config\powershell\_wado.ps1"
```

</details>

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
- Version: `<major><minor>`
- Index: If multiple containers of the same version and type are used, a zero-based index is added to the name.

## Ports

If not specified otherwise, the standalone and domain controller containers publish their HTTP and management ports
based on the WildFly version:

- 8080 → 8`<major><minor>`
- 9900 → 9`<major><minor>`

So for WildFly 34, the port mappings are 8340 and 9340, and for WildFly 26.1, the port mappings are 8261 and 9261.
If multiple containers of the same version are used, the port is increased by one from the second container onwards.

```shell
wado start 26.1,28..30,2x32,3x35
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

# Commands

Currently, the following commands are supported:

```shell
Command line tool to build and run WildFly containers in different versions and operation modes.

Usage: wado <COMMAND>

Commands:
  build     Build WildFly images
  start     Start a standalone server
  stop      Stop a standalone server
  dc        Start and stop a domain controller
  hc        Start and stop a host controller
  topology  Start and stop a topology defined as YAML
  images    List all available standalone, domain and host controller images
  ps        List running images
  console   Open the management console
  cli       Connect to the CLI
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

> [!IMPORTANT]
> Most commands require `podman` to be present with `docker` as a fallback.
> The `console` command opens the default web browser and the `cli` command requires a JVM.

## Build

If not specified otherwise, the build command builds standalone, domain, and host controller images based on the
official WildFly images. The images contain specific [modifications](#image-modifications) and a management user with a
predefined username and password.

Predefined images for
all [supported versions](https://github.com/hpehl/wildfly-container-versions?tab=readme-ov-file#supported-versions)
are available at https://quay.io/organization/wado. If you want to change the username and password, you can build your
own local image.

```shell
Build WildFly images

Usage: wado build [OPTIONS] <wildfly-version>

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
wado build 34
wado build 34 --username alice --password "Admin#70365"
wado build 10,23,34 --standalone
wado build 20..29 --domain
wado build 10,20..29,34
wado build .. --chuncks 5
```

## Standalone

### Start

```shell
Start a standalone server

Usage: wado start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the standalone server

Options:
  -n, --name <name>              The name of the standalone server [default: wado-sa-<major><minor>].
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

```shell
Stop a standalone server

Usage: wado stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the standalone server [default: wado-sa-<major><minor>]
  -a, --all          Stop all running standalone servers. If specified with a version,
                     stop all running standalone servers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

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

```shell
Start a domain controller

Usage: wado dc start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the domain controller

Options:
  -n, --name <name>              The name of the domain controller [default: wado-dc-<major><minor>].
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

```shell
Stop a domain controller

Usage: wado dc stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the domain controller [default: wado-dc-<major><minor>]
  -a, --all          Stop all running domain controllers. If specified with a version,
                     stop all running domain controllers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

```shell
wado dc stop 34
wado dc stop 30..35
wado dc stop 34 --name foo
wado dc stop 34 --all
wado dc stop --all
```

### Host Controller

#### Start

```shell
Start a host controller

Usage: wado hc start [OPTIONS] <wildfly-version> [-- [wildfly-parameters]...]

Arguments:
  <wildfly-version>        A single WildFly version or version range
  [wildfly-parameters]...  Parameters passed to the domain controller

Options:
  -n, --name <name>
          The name of the host controller [default: wado-hc-<major><minor>].
          Not allowed when multiple versions are specified.
  -d, --domain-controller <domain-controller>
          The name of the domain controller [default: wado-dc-<major><minor>].
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

```shell
Stop a host controller

Usage: wado hc stop [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range

Options:
  -n, --name <name>  The name of the host controller [default: wado-hc-<major><minor>]
  -a, --all          Stop all running host controllers. If specified with a version,
                     stop all running host controllers of that version.
  -h, --help         Print help
  -V, --version      Print version
```

**Examples**

```shell
wado hc stop 34
wado hc stop 30..35
wado hc stop 34 --name foo
wado hc stop 34 --all
wado hc stop --all
```

### Topology

> [!WARNING]
> The topology commands are not yet implemented.
> You can work around with the `dc` and `hc` commands though:
>
> ```shell
> wado dc start 35 -n dc -s s1,s2,s3,s4:osg,s5:osg
> wado hc start 32,33,2x35 -d dc -s s1,s2,s3:osg
> wado console 35
> ```
> Open http://localhost:9350/console/index.html#runtime;path=domain-browse-by~topology

#### Start

```shell
Start a topology

Usage: wado topology start <setup>

Arguments:
  <setup>  The topology setup

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### Stop

```shell
Stop a topology

Usage: wado topology stop <setup>

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

## Images

```shell
List all available standalone, domain and host controller images

Usage: wado images

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## PS

```shell
List running standalone, domain and host controller containers

Usage: wado ps [OPTIONS]

Options:
      --standalone  List standalone containers only
      --domain      List domain controller and host controller containers only
  -h, --help        Print help
  -V, --version     Print version
```

## Management Clients

### Console

```shell
Open the management console

Usage: wado console [OPTIONS] [wildfly-version]

Arguments:
  [wildfly-version]  A single WildFly version or version range.
                     If omitted the console is opened for all running standalone and domain controller containers.

Options:
  -n, --name <name>              The name of the standalone server or domain controller [default: wado-sa|dc-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port. Not allowed when multiple versions are specified.
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
wado console
wado console 34
wado console 30..35
wado console 34 --management 9990
```

### CLI

If not already present, this command downloads the `wildfly-cli-client.jar` and `jboss-cli.xml` of the specified version
to the `$TMPDIR`.

```shell
Connect to the CLI

Usage: wado cli [OPTIONS] [wildfly-version] [-- [cli-parameters]...]

Arguments:
  [wildfly-version]    A single WildFly version.
                       Can be omitted if only one standalone or domain controller is running.
  [cli-parameters]...  Parameters passed to the CLI

Options:
  -n, --name <name>              The name of the standalone server or domain controller [default: wado-sa|dc-<major><minor>].
                                 Not allowed when multiple versions are specified.
  -m, --management <management>  The published management port
  -u, --username <username>      The username to connect to the CLI [default: admin]
  -p, --password <password>      The password to connect to the CLI [default: admin]
  -h, --help                     Print help
  -V, --version                  Print version
```

**Examples**

```shell
wado cli
wado cli 34
wado cli 34 -- --command "/subsystem=logging/console-handler=CONSOLE:write-attribute(name=level,value=DEBUG)"
```
