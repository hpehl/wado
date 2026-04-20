# Code Review Issues

Review date: 2026-04-20

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 1 | FIXED |
| HIGH | 4 | 3 FIXED, 1 WON'T FIX |
| MEDIUM | 8 | 4 FIXED, 4 OPEN |
| LOW | 3 | 1 FIXED, 2 OPEN |

## CRITICAL

### Command Injection in CLI/Bootstrap Operations

**Files:** `src/cli.rs:112-118`, `src/resources.rs:66-72`

User-provided parameters and `$WADO_BOOTSTRAP_OPERATIONS` are passed to shell commands without sanitization. A malicious value like `"; rm -rf / #"` in bootstrap operations would execute arbitrary commands.

**Fix:** Validate bootstrap operations against JBoss CLI syntax, or pass via stdin instead of command-line args.

**Status:** FIXED — Added `is_valid_cli_operation()` validation in `src/args.rs` that rejects operations not starting with `/` or `:`.

## HIGH

### Hardcoded Default Credentials

**Files:** `src/app.rs:26,31,249,254,374,379`, `src/topology.rs:145-146`

Default "admin"/"admin" credentials throughout. Consider requiring explicit specification or env var fallback (`WADO_USERNAME`/`WADO_PASSWORD`).

**Status:** Won't fix (intentional for local dev use)

### Missing Path Validation

**Files:** `src/args.rs:50-62`

File paths from `--cli` argument read without validation. Could read unintended files via path traversal.

**Fix:** Validate or canonicalize paths before reading.

**Status:** FIXED — Added `canonicalize()` call before reading CLI file paths.

### Large Function

**Files:** `src/build/dev/source.rs:119-229`

`clone_and_build_repo_inner` is 111 lines with complex async logic. Should be decomposed.

**Fix:** Extract into smaller focused functions.

### Errors Silently Swallowed

**Files:** `src/build/dev/mod.rs:131-132,233-235`, `src/build/dev/source.rs:222,366`

Multiple `.ok()` calls suppress errors without logging, making debugging hard.

**Fix:** Log warnings instead of silently swallowing errors.

**Status:** FIXED — Changed `.ok()` calls to `eprintln!` warnings; `remove_volume` now returns `Result`.

## MEDIUM

- **Missing HTTP timeout** — `src/cli.rs:137` — `reqwest::get()` has no timeout — **FIXED**
- **Race condition** in container name generation — `src/container.rs:184-207`
- **Magic number** `100` — `src/args.rs:96` — should be a named constant — **FIXED**
- **Unbounded log file** — `src/build/dev/source.rs:156-212` — no size limit
- **Regex not cached** — `src/build/dev/source.rs:445` — recompiled on each call — **FIXED**
- **Symlink no fallback** for non-unix/windows platforms — `src/build/dev/mod.rs:337-343`

## LOW

- Inconsistent error message formatting (case, punctuation) — **FIXED** (java/container runtime messages)
- `HashMap` could be `BTreeMap` for deterministic ordering in `topology.rs`
- Missing documentation for public API functions
