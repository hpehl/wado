# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fix cargo-release double confirmation prompt by adding `--no-confirm` flag
- Fix tag not being pushed to GitHub by setting explicit `push-remote = "origin"`
- Fix double `v` prefix in tag name (`vv0.4.11` → `v0.4.11`) by removing
  redundant `tag-prefix` from `release.toml`

## [0.4.11] - 2026-04-23

### Changed

- Replace manual version bumping, changelog editing, and git operations in
  `release.sh` with `cargo-release` driven by `release.toml`
- Add `[Unreleased]` section to changelog for collecting notes between releases

## [0.4.10] - 2026-04-23

### Fixed

- Fix the name of the WildFly them when building HAL for the dev containers

## [0.4.9] - 2026-04-22

### Fixed

- Fix first container getting an unnecessary `-0` suffix when starting multiple
  instances of the same version (e.g. `wado start 2x39` now produces
  `wado-sa-390` and `wado-sa-390-1` instead of `wado-sa-390-0` and
  `wado-sa-390-1`)

## [0.4.8] - 2026-04-22

### Changed

- Support `dev` and dotted versions (e.g. `26.1`) in topology YAML files,
  aligning with the `wildfly-version` parameter accepted by other commands

## [0.4.7] - 2026-04-22

### Changed

- Default server group to `main-server-group` in topology YAML files when
  `group` is not specified

## [0.4.6] - 2026-04-22

### Fixed

- Sort dev containers last in `wado images` output instead of first
- Display "dev" instead of raw version string for dev containers in
  `wado images` output

## [0.4.5] - 2026-04-22

### Added

- Dynamic topology name completion for `wado stop topology`, suggesting
  names of currently running topologies

### Changed

- Move all subcommand implementations into a `command` module
- Group all shell completion logic into a `completion` module
- Move shared container helpers (`resolve_instances`, `stop_command`) into
  `container/lifecycle` and rename to `prepare_instances` and
  `stop_containers_by_server_type` for clarity
- Move `create_secret` and `add_servers` into `container/command`
- Split `wildfly.rs` and `container.rs` into module directories

## [0.4.4] - 2026-04-21

### Changed

- Auto-apply socket-binding port offsets to servers in topology YAML files when
  offsets are not explicitly specified, matching the existing behavior of
  `--server` in `wado dc start` and `wado hc start`
- Move `apply_offsets` and `DEFAULT_SERVER_OFFSET` from `args` to `wildfly`
  module for shared use across CLI and topology code paths

## [0.4.3] - 2026-04-21

### Added

- Show topology and config columns in `wado ps` output
- Attach config label (`org.wildfly.wado.config`) to containers at start time

### Changed

- Centralize container name and port adjustment into `StartSpec` and
  `resolve_start_specs`, replacing duplicated logic across standalone, domain
  controller, host controller, and topology start commands
- Name and port adjustment are now independent — providing `--name` no longer
  prevents port auto-adjustment, and providing `--http` or `--management` no
  longer prevents name auto-adjustment
- Per-port granularity: `--http`, `--management`, and `--offset` independently
  mark individual ports as user-provided
- Centralize container label management into a `Label` enum replacing scattered
  string constants across the codebase
- Extract `extract_config` helper to parse `-c` / `--server-config=` from
  parameters

### Fixed

- Fix port conflicts when starting different server types for the same WildFly
  version (e.g. `wado start 39` then `wado dc start 39`). Port offset now
  counts running standalone and domain controller containers (which publish
  ports), excluding host controllers. Container name indexing only considers
  containers of the same type.

## [0.4.2] - 2026-04-20

### Changed

- Cache HAL jar regex compilation via `LazyLock` instead of recompiling per call
- Extract magic number `100` to `DEFAULT_SERVER_OFFSET` constant
- Improve error messages for missing `java`, `podman`, and `docker` commands
- Log warnings instead of silently swallowing errors during volume removal,
  container cleanup, and log file deletion
- Decompose `clone_and_build_repo_inner` into smaller focused functions
- Use `BTreeMap` for deterministic host controller ordering in topology
- Add doc comments to public API types and traits
- Cap build log files at 10 MB to prevent unbounded disk usage
- Add file copy fallback for symlinks on non-unix/windows platforms

### Security

- Validate bootstrap operations to only accept valid JBoss CLI operations
  (must start with `/` or `:`) and skip comments/empty lines from CLI files
- Canonicalize `--cli` file paths before reading to prevent path traversal

### Fixed

- Add a 30-second timeout to CLI jar/config HTTP downloads
- Check for container name conflicts before starting to prevent race condition
  in concurrent `wado start` invocations

## [0.4.1] - 2026-04-20

### Fixed

- Fix `wado ps` not listing running containers due to swapped identifier and
  container ID arguments when parsing container instance data

## [0.4.0] - 2026-04-20

### Added

- Topology support: define multi-host domain topologies in YAML and start/stop
  them as a unit with `topology start` and `topology stop`
- Topology containers are labeled for discovery, so `topology stop` can find
  running containers by topology name without requiring the setup file
- Support for unnamed hosts in topology setups with automatic container name
  and port assignment

### Changed

- Refactor `container_ps` into reusable `ps_instances` helper
- Extract `build_server_map` and `build_host_controllers` for clearer topology
  orchestration

## [0.3.5] - 2026-04-18

### Fixed

- Always pull latest dev container image instead of using stale local cache

## [0.3.4] - 2026-04-18

### Added

- Dynamic name completion for `--name` option on `stop`, `dc stop`, `hc stop`,
  `console`, and `cli` commands, suggesting running container names
- Dynamic name completion for `--domain-controller` option on `hc start`,
  suggesting running domain controller names

## [0.3.3] - 2026-04-17

### Added

- Context-aware version completion: `stop`, `dc stop`, `hc stop`, `console`, and
  `cli` now only suggest running container versions

## [0.3.2] - 2026-04-17

### Added

- Add `dev` to shell completion for wildfly-version arguments
- Context-aware version completion: `stop`, `dc stop`, `hc stop`, `console`, and
  `cli` now only suggest running container versions

## [0.3.1] - 2026-04-17

### Fixed

- Fix dev container push using `podman push` instead of `podman manifest push`

## [0.3.0] - 2026-04-17

### Added

- Dev container support for building from WildFly and HAL source branches
- Auto-increment container names and ports when starting duplicate versions (e.g., `wado start dev` twice, or
  `wado start dev` then `wado start 2xdev`)
- Add comma-separated version completion tests

### Changed

- Extract shared build helpers into `build/common.rs` and unify build module structure
- Replace mutation with immutable patterns in container port mapping and image listing
- Flatten `parse_server()` deep nesting with sequential consumption
- Deduplicate container runtime detection and `AdminContainer` constructors
- Unify start instance async boilerplate with generic `run_instances()` function
- Consolidate domain model struct impls with `ContainerConfig` trait and macro
- Consolidate six Dockerfile templates into one parameterized template
- Simplify `find_suggestions()` by extracting prefix parsing, version helpers, and removing repeated tuple construction
- Replace ~1000-char `NO_AUTH` constant with `sed_remove_auth!()` macro to define sed patterns once
- Write secret values directly to stdin instead of spawning an `echo` process in `create_secret()`

### Fixed

- Eliminate shell injection surface in multi-platform build commands

## [0.2.13] - 2026-04-13

### Changed

- Replace static shell completions with dynamic completions

## [0.2.12] - 2026-02-13

### Added

- Upgrade to WildFly 39.0.1

## [0.2.11] - 2026-02-13

### Added

- Upgrade to WildFly 39.0.1

## [0.2.10] - 2026-02-13

### Added

- Upgrade to WildFly 39.0.1

## [0.2.9] - 2026-01-19

### Added

- Upgrade to WildFly 39.0.0

## [0.2.8] - 2025-11-18

### Added

- Upgrade to WildFly 38.0.1

## [0.2.7] - 2025-10-21

### Added

- Upgrade to WildFly 38.0.0

## [0.2.6] - 2025-09-11

### Added

- Publish a library

## [0.2.5] - 2025-09-09

### Added

- Upgrade to WildFly 37.0.1

## [0.2.4] - 2025-08-05

### Added

- Add support for WildFly 37

## [0.2.3] - 2025-06-06

### Changed

- Refactor wildfly-version completion

### Fixed

- #6: Fix command completions

## [0.2.2] - 2025-05-20

### Fixed

- Exclude applying patches during build when on Windows

## [0.2.1] - 2025-05-20

### Added

- #5: Add dynamic shell completion for `wildfly-versions`

## [0.2.0] - 2025-05-19

### Changed

- Rename `wfadm` → `wado` (https://martinfowler.com/bliki/TwoHardThings.html)

## [0.1.0] - 2025-05-19

### Changed

- Rename `waco` → `wfadm`

## [0.0.12] - 2025-05-15

### Upgrades

- Bump `wildfly_container_versions` to 0.2.1

## [0.0.11] - 2025-05-04

### Added

- Images sub-command (#2)

### Changed

- Change container name from `waco-<version>-<type>[-index]` tp `waco-<type>-<version>[-index]`

## [0.0.10] - 2025-04-15

### Fixed

- Fix `dc start` with multiple versions and missing domain controller option

## [0.0.9] - 2025-04-15

### Fixed

- Fix `dc start` with multiple versions and missing domain controller option

## [0.0.8] - 2025-04-15

### Added

- Fix #4: WildFly version is now optional for the `console` and `cli` sub commands

## [0.0.7] - 2025-04-14

### Fixed

- Fix the release workflow

## [0.0.6] - 2025-04-14

### Added

- Include shell completions in release and brew formula

## [0.0.5] - 2025-04-13

### Fixed

- Fix the release workflow

## [0.0.4] - 2025-04-13

### Changed

- Change the release workflow

## [0.0.3] - 2025-04-12

### Fixed

- Fix the release workflow

## [0.0.2] - 2025-04-12

### Added

- Fix #1: Add support for docker
- Fix #3: Add support for windows

## [0.0.1] - 2025-04-12

### Added

- First release 🎉

[Unreleased]: https://github.com/hpehl/wado/compare/v0.4.11...HEAD
[0.4.11]: https://github.com/hpehl/wado/compare/v0.4.10...v0.4.11

[0.4.10]: https://github.com/hpehl/wado/compare/v0.4.9...v0.4.10

[0.4.9]: https://github.com/hpehl/wado/compare/v0.4.8...v0.4.9

[0.4.8]: https://github.com/hpehl/wado/compare/v0.4.7...v0.4.8

[0.4.7]: https://github.com/hpehl/wado/compare/v0.4.6...v0.4.7

[0.4.6]: https://github.com/hpehl/wado/compare/v0.4.5...v0.4.6

[0.4.5]: https://github.com/hpehl/wado/compare/v0.4.4...v0.4.5

[0.4.4]: https://github.com/hpehl/wado/compare/v0.4.3...v0.4.4

[0.4.3]: https://github.com/hpehl/wado/compare/v0.4.2...v0.4.3

[0.4.2]: https://github.com/hpehl/wado/compare/v0.4.1...v0.4.2

[0.4.1]: https://github.com/hpehl/wado/compare/v0.4.0...v0.4.1

[0.4.0]: https://github.com/hpehl/wado/compare/v0.3.5...v0.4.0

[0.3.5]: https://github.com/hpehl/wado/compare/v0.3.4...v0.3.5

[0.3.4]: https://github.com/hpehl/wado/compare/v0.3.3...v0.3.4

[0.3.3]: https://github.com/hpehl/wado/compare/v0.3.2...v0.3.3

[0.3.2]: https://github.com/hpehl/wado/compare/v0.3.1...v0.3.2

[0.3.1]: https://github.com/hpehl/wado/compare/v0.3.0...v0.3.1

[0.3.0]: https://github.com/hpehl/wado/compare/v0.2.13...v0.3.0

[0.2.13]: https://github.com/hpehl/wado/compare/v0.2.12...v0.2.13

[0.2.12]: https://github.com/hpehl/wado/compare/v0.2.11...v0.2.12

[0.2.11]: https://github.com/hpehl/wado/compare/v0.2.10...v0.2.11

[0.2.10]: https://github.com/hpehl/wado/compare/v0.2.9...v0.2.10

[0.2.9]: https://github.com/hpehl/wado/compare/v0.2.8...v0.2.9

[0.2.8]: https://github.com/hpehl/wado/compare/v0.2.7...v0.2.8

[0.2.7]: https://github.com/hpehl/wado/compare/v0.2.6...v0.2.7

[0.2.6]: https://github.com/hpehl/wado/compare/v0.2.5...v0.2.6

[0.2.5]: https://github.com/hpehl/wado/compare/v0.2.4...v0.2.5

[0.2.4]: https://github.com/hpehl/wado/compare/v0.2.3...v0.2.4

[0.2.3]: https://github.com/hpehl/wado/compare/v0.2.2...v0.2.3

[0.2.2]: https://github.com/hpehl/wado/compare/v0.2.1...v0.2.2

[0.2.1]: https://github.com/hpehl/wado/compare/v0.2.0...v0.2.1

[0.2.0]: https://github.com/hpehl/wado/compare/v0.1.0...v0.2.0

[0.1.0]: https://github.com/hpehl/wado/compare/v0.0.12...v0.1.0

[0.0.12]: https://github.com/hpehl/wado/compare/v0.0.11...v0.0.12

[0.0.11]: https://github.com/hpehl/wado/compare/v0.0.10...v0.0.11

[0.0.10]: https://github.com/hpehl/wado/compare/v0.0.9...v0.0.10

[0.0.9]: https://github.com/hpehl/wado/compare/v0.0.8...v0.0.9

[0.0.8]: https://github.com/hpehl/wado/compare/v0.0.7...v0.0.8

[0.0.7]: https://github.com/hpehl/wado/compare/v0.0.6...v0.0.7

[0.0.6]: https://github.com/hpehl/wado/compare/v0.0.5...v0.0.6

[0.0.5]: https://github.com/hpehl/wado/compare/v0.0.4...v0.0.5

[0.0.4]: https://github.com/hpehl/wado/compare/v0.0.3...v0.0.4

[0.0.3]: https://github.com/hpehl/wado/compare/v0.0.2...v0.0.3

[0.0.2]: https://github.com/hpehl/wado/compare/v0.0.1...v0.0.2

[0.0.1]: https://github.com/hpehl/wado/releases/tag/v0.0.1
