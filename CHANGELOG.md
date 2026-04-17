# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-04-17

### Added

- Add `dev` to shell completion for wildfly-version arguments
- Context-aware version completion: `stop`, `dc stop`, `hc stop`, `console`, and `cli` now only suggest running container versions

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
