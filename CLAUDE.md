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

The crate is a binary (`src/main.rs`).

### Key Modules

- **`main.rs`** - Entry point. Builds the full CLI with clap, wires up argument parsers and version completers, dispatches subcommands. Uses `#[tokio::main]` for async runtime.
- **`app.rs`** - Defines the CLI structure (subcommands, args, flags) using clap's builder API. Separated from `main.rs` to keep the parser/completer wiring out of the app definition.
- **`wildfly/`** - Core domain model (module directory): `ServerType` (sa/dc/hc), `AdminImage`, `Ports`, `StandaloneInstance`, `DomainController`, `HostController`, `ContainerInstance`, `Server`, `ManagementClient`. Contains the `Server::parse_server()` parser and all unit tests.
- **`container/`** - Container runtime interaction (module directory): orchestrates podman/docker commands (run, stop, ps, network, inspect), detects runtime via `which`, handles auto-incrementing container names/ports for duplicate versions. All container operations are async (tokio).
- **`command/`** - Subcommand implementations (module directory): `standalone.rs`, `dc.rs`, `hc.rs`, `build/` (image builds with Handlebars templates), `cli.rs` (JBoss CLI), `console.rs` (management console), `ps.rs`, `images.rs`, `versions.rs`, `push.rs`, `topology/` (YAML-based domain topologies), `update.rs`, `completions.rs`, `lifecycle.rs`.
- **`progress.rs`** - Progress bar utilities for long-running container operations.
- **`completion/`** - Shell completion logic (module directory): version completers, running container completers, topology completers.
- **`resources.rs`** - Embedded Dockerfile templates and entrypoint shell scripts for all three server types (standalone, domain controller, host controller).
- **`constants.rs`** - Container naming, labels, environment variables, sed expressions for XML config modifications.
- **`args.rs`** - Shared argument extraction helpers used across subcommands.
- **`label.rs`** - OCI label helpers for filtering and formatting container metadata.

### External Dependency

`wildfly_meta` crate provides `WildFlyImage`, `WildFlyImageRegistry`, and version expression parsing (`parse_wildfly_image()`, `parse_wildfly_images()`). The registry is loaded from `~/.config/wildfly-meta/` and can be updated via `wado update`.

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
