# Code Review Issues

Found during a full-project code review on 2026-04-16 using Claude Code.
The review covered all Rust source files in `src/` and focused on code duplication,
security, and code quality.

## Open

### SEC-2: Hardcoded default credentials (LOW)

**File:** `src/app.rs:27-32`

`admin/admin` used as defaults for username/password arguments. Acceptable for
local development containers but worth documenting.

### QUAL-2: `find_suggestions()` complexity (MEDIUM)

**File:** `src/wildfly_version.rs:17-103`

87 lines with 7 branches, most building the same `(prefix, token, filtered_versions)`
tuple with minor filtering variations.

### QUAL-5: `NO_AUTH` constant (LOW)

**File:** `src/constants.rs:16`

~1000-character single line with the same sed pattern repeated 3 times (for three
different XML attributes). Hard to read and maintain.

### SEC-3: Shell injection surface in `build_maven_command()` (MEDIUM)

**File:** `src/build/dev/source.rs:32-36`

`format!()` interpolates `branch` and `repo_url` into a shell script string
passed to `sh -c`. No sanitization of shell metacharacters. Blast radius is
limited since the command runs inside a throwaway container.

### QUAL-8: Unnecessary `echo` process in `create_secret()` (LOW)

**File:** `src/hc.rs:162-167`

Spawns an `echo -n` process to pipe a value to `podman secret create`. Could
write directly to stdin, eliminating the extra process. Not a security issue
since `Command::arg()` is safe from shell injection.

### QUAL-6: TODO comment (LOW)

**File:** `src/wildfly_version.rs:388`

```rust
// TODO Test commas
```
