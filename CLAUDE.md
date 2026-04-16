# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`wado` (**W**ildFly **ad**min c**o**ntainers) is a Rust CLI tool for building and running WildFly application server containers across multiple versions and operation modes (standalone, domain controller, host controller). It uses podman (with docker fallback) to manage containers, and images are hosted at quay.io/wado.

## Build & Development Commands

```shell
cargo build                    # Debug build
cargo build --release          # Release build (uses LTO)
cargo install --path .         # Install to ~/.cargo/bin/
cargo test                     # Run all tests
cargo test wildfly::tests      # Run tests in wildfly module
cargo clippy                   # Lint
cargo fmt                      # Format
```

## Architecture

The crate is both a library (`src/lib.rs`) and a binary (`src/main.rs`).

### Key Modules

- **`main.rs`** - Entry point. Builds the full CLI with clap, wires up argument parsers and version completers, dispatches subcommands. Uses `#[tokio::main]` for async runtime.
- **`app.rs`** - Defines the CLI structure (subcommands, args, flags) using clap's builder API. Separated from `main.rs` so `lib.rs` can reuse it without the parser/completer wiring.
- **`wildfly.rs`** - Core domain model: `ServerType` (sa/dc/hc), `AdminContainer`, `Ports`, `StandaloneInstance`, `DomainController`, `HostController`, `ContainerInstance`, `Server`, `ManagementClient`. Contains the `Server::parse_server()` parser and all unit tests.
- **`container.rs`** - Orchestrates podman/docker commands: build, run, stop, ps, network, inspect. Detects container runtime via `which`. All container operations are async (tokio).
- **`build.rs`** - Image build logic. Renders Dockerfiles from Handlebars templates, manages secrets for credentials, supports chunked parallel builds.
- **`resources.rs`** - Embedded Dockerfile templates and entrypoint shell scripts for all three server types (standalone, domain controller, host controller).
- **`constants.rs`** - Container naming, labels, environment variables, sed expressions for XML config modifications.
- **`args.rs`** - Shared argument extraction helpers used across subcommands.
- **`wildfly_version.rs`** - Shell completion logic for WildFly version arguments.

### External Dependency

`wildfly_container_versions` crate provides the `WildFlyContainer` type and `VERSIONS` map with all supported WildFly versions, their base images, platform support, and version metadata. Version parsing (`WildFlyContainer::enumeration()`, `WildFlyContainer::version()`) lives there.

### Patterns

- Container runtime abstraction: `container_command()` checks for podman first, falls back to docker.
- Parallel container operations use `tokio::task::JoinSet` with `MultiProgress` for concurrent progress bars.
- Dockerfiles are Handlebars templates rendered at build time with version-specific data (e.g., `primary`/`master` naming changed at WildFly 27).
- Container naming convention: `wado-{sa|dc|hc}-{major}{minor}[-index]`.
- Port mapping convention: HTTP `8{major}{minor}`, management `9{major}{minor}`.

## CI

- `.github/workflows/verify.yml` - Build and test verification
- `.github/workflows/release.yml` - Release workflow
- `.github/dependabot.yml` - Dependency updates for cargo and GitHub Actions
