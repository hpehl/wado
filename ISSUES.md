# Code Review Issues

Found during a full-project code review on 2026-04-16 using Claude Code.
The review covered all Rust source files in `src/` and focused on code duplication,
security, and code quality. Issues marked **(RESOLVED)** were fixed in the same session.

## Resolved

### DUP-1: `podman_build()` vs `dev_podman_build()` (HIGH) **(RESOLVED)**

**Files:** `src/build.rs`, `src/dev/mod.rs`

Both functions duplicated entrypoint writing, Handlebars template rendering, and
multi-platform build command construction (~80 lines).

**Fix:** Extracted `write_entrypoint()`, `base_template_data()`, `render_dockerfile()`,
and `container_build_command()` as shared helpers in `build.rs`. Both `podman_build()`
and `dev_podman_build()` now delegate to these helpers.

### DUP-2: Verbose build loops (HIGH) **(RESOLVED)**

**Files:** `src/build.rs:start_builds_verbose()`, `src/dev/mod.rs:build_containers_verbose()`

Identical verbose build loops (~40 lines each) with the same success/error output pattern.

**Fix:** Extracted `run_builds_verbose()` in `build.rs` that takes a closure for the
build command. Both callers now delegate with a one-liner.

### DUP-3: Stop functions (MEDIUM) **(RESOLVED)**

**Files:** `src/standalone.rs`, `src/dc.rs`, `src/hc.rs`

All three stop functions were identical except for the `ServerType` variant (~10 lines x3).

**Fix:** Extracted `stop_command(server_type, matches)` in `args.rs`. Each stop function
now delegates in a single line.

### DUP-4: Multi-version validation guards (MEDIUM) **(RESOLVED)**

**Files:** `src/standalone.rs`, `src/dc.rs`

Same four `matches.contains_id()` checks with identical bail messages (~10 lines x2).

**Fix:** Extracted `validate_single_version(matches, &[...])` in `args.rs`. Each caller
now validates with a single line.

## Open

### DUP-5: Start instance async boilerplate (MEDIUM)

**Files:** `src/standalone.rs:68-113`, `src/dc.rs:72-121`, `src/hc.rs:90-158`

All three `start_instances` functions follow the same async pattern: create
`MultiProgress` + `JoinSet`, call `container_network()`, loop with `Progress` +
spawn child + stderr reader, then `summary()`. The core loop body is repeated
three times (~120 lines total). The command setup differs per server type, making
a clean extraction non-trivial.

### DUP-6: Container runtime detection x3 (MEDIUM)

**File:** `src/container.rs:285-317`

Three functions independently probe for podman/docker:
- `verify_container_command()` returns `PathBuf`
- `container_command()` returns `Command`
- `container_command_name()` returns `&'static str`

**Suggestion:** A single `detect_runtime() -> Result<(&'static str, PathBuf)>` that
the others wrap.

### DUP-7: Domain model struct boilerplate (MEDIUM)

**File:** `src/wildfly.rs:241-340`

`StandaloneInstance`, `DomainController`, `HostController` share `admin_container`
and `name` fields, identical `HasWildFlyContainer` impls, and the same `copy()` pattern.

### DUP-8: AdminContainer constructors (LOW)

**File:** `src/wildfly.rs:59-84`

`standalone()`, `domain_controller()`, `host_controller()` differ only in the
`ServerType` variant. Could be a single `fn new(wc, server_type)`.

### DUP-9: Dockerfile template preamble (LOW)

**File:** `src/resources.rs`

The three dev Dockerfiles share identical 8-line preambles. The DC and HC dev
Dockerfiles differ only in `host-primary.xml` vs `host-secondary.xml`. Same
pattern exists for stable Dockerfiles.

### DUP-10: Template data HashMap construction (MEDIUM) **(RESOLVED as part of DUP-1)**

Shared `base_template_data()` now handles the common keys.

### SEC-1: Shell injection surface in build commands (MEDIUM)

**Files:** `src/build.rs:container_build_command()`, formerly also `src/dev/mod.rs`

Multi-platform builds construct shell commands via `format!()` and pass them to
`sh -c`. While interpolated values are currently derived from controlled sources,
any future change introducing user-controlled values into paths or image names
could create an injection vector.

**Suggestion:** Avoid the shell wrapper. Podman/buildah can do multi-platform
builds without `sh -c` by using `--manifest` directly.

### SEC-2: Hardcoded default credentials (LOW)

**File:** `src/app.rs:27-32`

`admin/admin` used as defaults for username/password arguments. Acceptable for
local development containers but worth documenting.

### QUAL-1: `parse_server()` deep nesting (HIGH)

**File:** `src/wildfly.rs:422-496`

75 lines with 6 levels of nesting and duplicated conditional patterns. The
`parts[0].eq_ignore_ascii_case("start")` / `parts[0].parse::<u16>()` /
`parts.remove(0)` pattern appears three times.

### QUAL-2: `find_suggestions()` complexity (MEDIUM)

**File:** `src/wildfly_version.rs:17-103`

87 lines with 7 branches, most building the same `(prefix, token, filtered_versions)`
tuple with minor filtering variations.

### QUAL-3: Mutation of `ContainerInstance` (MEDIUM)

**File:** `src/container.rs:51-73`

`container_ports()` takes `&mut ContainerInstance` and mutates `.ports` in place.
Should return a new `ContainerInstance` with ports set (immutable pattern).

### QUAL-4: Mutation of `HashMap<String, AdminContainer>` (MEDIUM)

**File:** `src/image.rs:45-79`

`local_images()` and `images_in_use()` mutate `AdminContainer` structs inside
the HashMap by setting `local_image` and `in_use` fields in place.

### QUAL-5: `NO_AUTH` constant (LOW)

**File:** `src/constants.rs:16`

~1000-character single line with the same sed pattern repeated 3 times (for three
different XML attributes). Hard to read and maintain.

### QUAL-6: TODO comment (LOW)

**File:** `src/wildfly_version.rs:388`

```rust
// TODO Test commas
```
