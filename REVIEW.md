# Codebase Review & Simplification Findings

This document collects findings from a comprehensive code review and simplification analysis of the wado codebase. Each finding is self-contained so you can address them one by one.

---

## ~~1. `wildfly.rs` exceeds 800 lines — split into module directory~~ FIXED

**File:** `src/wildfly.rs` (832 lines)
**Severity:** HIGH

This file contains 10+ structs/enums, a trait, a macro, and ~200 lines of tests. It mixes several distinct concerns:

- **Server types & container config** (`ServerType`, `ContainerConfig` trait, `AdminContainer`)
- **Instance types** (`StandaloneInstance`, `DomainController`, `HostController`, `ContainerInstance`)
- **Server parsing & domain concepts** (`ServerGroup`, `Server`, `apply_offsets`, `DEFAULT_SERVER_OFFSET`)
- **Management client** (`ManagementClient` — CLI/console connection config)
- **Start resolution types** (`StartSpec`, `ResolvedStart`, `Ports`)

**Suggested split into `src/wildfly/` module directory:**

| New file | Contents |
|----------|----------|
| `mod.rs` | Re-exports, `ContainerConfig` trait, `impl_container_instance!` macro |
| `server_type.rs` | `ServerType` enum + `FromStr` |
| `admin_container.rs` | `AdminContainer` struct + `Ord` impl |
| `instance.rs` | `Ports`, `StandaloneInstance`, `DomainController`, `HostController`, `ContainerInstance` |
| `server.rs` | `ServerGroup`, `Server`, `apply_offsets`, `DEFAULT_SERVER_OFFSET` |
| `management.rs` | `ManagementClient` |
| `start.rs` | `StartSpec`, `ResolvedStart` |

This follows the same pattern you already applied successfully to `container/`.

---

## ~~2. `topology.rs` and `topology_model.rs` should be a module directory~~ FIXED

**Files:** `src/topology.rs` (230 lines), `src/topology_model.rs` (357 lines)
**Severity:** MEDIUM

These two files are tightly coupled — `topology.rs` imports from `topology_model.rs`, and nothing else uses `topology_model.rs`. Grouping them as `src/topology/` would mirror the `container/` and `build/` patterns.

**Suggested structure:**

| New file | Contents |
|----------|----------|
| `mod.rs` | Re-exports `topology_start`, `topology_stop` |
| `model.rs` | `TopologySetup`, `HostSetup`, `ServerSetup` + tests (current `topology_model.rs`) |
| `start.rs` | `topology_start`, `build_hc_specs`, `build_server_map`, `start_topology` |
| `stop.rs` | `topology_stop`, `resolve_topology_name`, `stop_topology` |

---

## ~~3. `standalone.rs`, `dc.rs`, `hc.rs` share a repetitive start pattern~~ FIXED

**Files:** `src/standalone.rs` (65 lines), `src/dc.rs` (74 lines), `src/hc.rs` (156 lines)
**Severity:** MEDIUM

All three follow the same pattern:
1. `verify_container_command()`
2. Extract versions, validate multiple versions
3. Build `StartSpec` list
4. `resolve_start_specs()`
5. Convert to typed instance
6. Call `start_instances()` with `run_instances()` inside

The boilerplate (steps 1-4) is nearly identical. Consider extracting a shared helper:

```rust
fn resolve_typed_instances<T>(
    matches: &ArgMatches,
    server_type: ServerType,
    restricted_options: &[&str],
    convert: impl Fn(ResolvedStart) -> T,
) -> anyhow::Result<Vec<T>>
```

This would reduce `standalone_start` and `dc_start` to ~15 lines each. `hc_start` has enough unique logic (domain controller lookup, secrets) that it benefits less, but could still use the shared validation.

**Alternative (lighter touch):** Leave as-is. The duplication is small (3 files, ~15 lines each), and each server type has subtle differences. The current code is very readable.

---

## ~~4. `create_secret` in `hc.rs` is used by `topology.rs` — move to `container/`~~ FIXED

**File:** `src/hc.rs:133` (`create_secret`), used by `src/topology.rs:9`
**Severity:** MEDIUM

`create_secret` creates a podman secret — this is a container-level operation. Currently `topology.rs` imports it from `hc.rs` via `crate::hc::create_secret`, which creates a circular dependency concern (topology -> hc).

**Suggestion:** Move `create_secret` to `src/container/command.rs` alongside the other container command builders. It fits the "low-level podman command" scope.

---

## 5. `args.rs` mixes argument extraction with command execution

**File:** `src/args.rs` (212 lines)
**Severity:** MEDIUM

Most functions in `args.rs` are pure argument extractors (pulling values from `ArgMatches`), but `stop_command()` (line 170) actually *executes* a stop operation. It calls `verify_container_command()`, `block_on()`, and `stop_instances()`.

**Suggestion:** Move `stop_command()` to `container/lifecycle.rs` or keep it but rename to make the side-effect obvious (e.g., `execute_stop_command`). Alternatively, inline it into each caller since it's only 10 lines and called from `standalone_stop`, `dc_stop`, and `hc_stop`.

---

## 6. `console.rs` and `cli.rs` share management client resolution logic

**Files:** `src/console.rs:18` (`get_management_clients`), `src/cli.rs:22-58`
**Severity:** LOW

Both files independently resolve which container(s) to connect to, with similar logic:
- Check for `--name` argument
- Check for `--wildfly-version`
- Fall back to querying all running containers

The logic isn't identical (cli needs exactly one, console allows multiple), but could share a common "resolve target container" helper.

**Suggestion:** Extract a shared function in `args.rs`:
```rust
pub fn resolve_management_targets(matches: &ArgMatches, allow_multiple: bool) -> Result<Vec<ManagementClient>>
```

This is a minor improvement — only pursue if you touch these files for other reasons.

---

## 7. `app.rs` has significant argument duplication across subcommands

**File:** `src/app.rs` (393 lines)
**Severity:** LOW

The `start` subcommands for standalone, dc, and hc repeat the same argument definitions:
- `wildfly-version`, `wildfly-parameters`, `name`, `http`, `management`, `offset` (sa/dc share these)
- `operations`, `cli` (all three share these)
- `server` (dc/hc share this with identical help text)

The `stop` subcommands also repeat `wildfly-version`, `name`, `all`.

**Suggestion:** Extract shared argument groups as functions:

```rust
fn start_args(cmd: Command) -> Command { /* version, name, http, management, offset */ }
fn stop_args(cmd: Command) -> Command { /* version, name, all */ }
fn server_arg(cmd: Command) -> Command { /* server definition */ }
fn operations_args(cmd: Command) -> Command { /* operations, cli */ }
```

This would cut `app.rs` by ~100 lines and eliminate drift risk between identical argument definitions.

---

## 8. `build/dev/source.rs` is 580 lines — review for extraction

**File:** `src/build/dev/source.rs` (580 lines)
**Severity:** MEDIUM

This is the second largest file and handles git cloning, maven builds, artifact extraction, and HAL integration. Without changing its scope, consider:

- Extracting maven-related helpers into `src/build/dev/maven.rs`
- Extracting artifact extraction (HAL jar, WildFly dist) into `src/build/dev/artifact.rs`

Note: I haven't read this file in full detail as it's in the `build/dev/` submodule which is already well-organized. Flag for later if you want to reduce its size.

---

## 9. Dead/questionable patterns

### 9a. `Ports::with_offset` is `#[cfg(test)]` only

**File:** `src/wildfly.rs:221-228`
**Severity:** LOW

`Ports::with_offset` exists only for test assertions. It's fine, but if you split `wildfly.rs` into a module directory, consider whether this should live in a test helper module instead.

### 9b. `AdminContainer.local_image` and `in_use` are mutable state on a "model" struct

**File:** `src/wildfly.rs:72-73`
**Severity:** LOW

`AdminContainer` is constructed with `local_image: false, in_use: false` everywhere, then these fields are set in `image.rs` by creating a new struct with `..ac`. This works but is a design smell — these flags are view-layer concerns, not core model properties.

**Suggestion:** Consider a separate `ImageStatus` wrapper or just compute these as part of the table rendering in `image.rs`, removing the fields from `AdminContainer` entirely.

---

## 10. Minor code quality items

### 10a. Inconsistent error handling in `container/command.rs`

**File:** `src/container/command.rs`
**Severity:** LOW

`container_command()` returns `anyhow::Result<Command>`, but `container_images_cmd()` (line 39), `container_run_cmd()` (line 73), and `container_stop_cmd()` (line 116) all call `.expect()` on it instead of propagating the error. These should either all return `Result` or document why panicking is acceptable.

### 10b. `container/resolve.rs:113` — `running_instance_counts` is `pub` but only used internally

**File:** `src/container/resolve.rs:113`
**Severity:** LOW

This function is `pub` but only called within the same file. It could be `pub(super)` or private.

### 10c. `ps_instances` and `container_ports` visibility

**File:** `src/container/query.rs:112,144`
**Severity:** LOW

Both are `pub(super)` which is correct, but `container_ports` is only used within `query.rs` itself. It could be private.

---

## Summary — priority order

| # | Finding | Severity | Effort |
|---|---------|----------|--------|
| 1 | Split `wildfly.rs` into module directory | HIGH | Medium |
| 4 | Move `create_secret` to `container/` | MEDIUM | Small |
| 2 | Group `topology` files into module directory | MEDIUM | Medium |
| 5 | Move `stop_command` out of `args.rs` | MEDIUM | Small |
| 3 | Reduce start-pattern duplication | MEDIUM | Medium |
| 8 | Consider splitting `build/dev/source.rs` | MEDIUM | Medium |
| 9b | Remove `local_image`/`in_use` from `AdminContainer` | LOW | Small |
| 7 | Extract shared arg builders in `app.rs` | LOW | Medium |
| 6 | Share management client resolution | LOW | Small |
| 10a | Fix inconsistent `expect()` vs `Result` | LOW | Small |
| 10b-c | Tighten visibility modifiers | LOW | Trivial |
| 9a | `Ports::with_offset` test-only concern | LOW | Trivial |
