---
name: rudo-gc-crate-switch
description: Switch rudo-gc between local workspace crate (for debugging rudo-gc) and crates.io published version. Use when you need to switch rudo-gc source for development or debugging.
disable-model-invocation: true
---

# rudo-gc Crate Source Switch

This skill documents how to switch the rudo-gc dependency in Rvue between:

1. **Local crate** – `learn-projects/rudo` submodule (for debugging rudo-gc itself, inspecting internals)
2. **crates.io** – published version (for normal development, releases, CI)

## Prerequisites

- **Local → crates.io**: No submodule required.
- **crates.io → Local**: The `learn-projects/rudo` submodule must be present. Initialize with:
  ```bash
  git submodule update --init learn-projects/rudo
  ```

---

## Switch TO crates.io (local → published)

Use this for normal development, releases, and when you don't need to modify rudo-gc.

### 1. Update workspace dependencies (`Cargo.toml`)

Replace the `[workspace.dependencies]` block:

```toml
# FROM (local):
[workspace.dependencies]
rudo-gc = { path = "learn-projects/rudo/crates/rudo-gc", features = ["test-util", "default", "tracing", "debug-suspicious-sweep", "tokio"] }
rudo-gc-derive = { path = "learn-projects/rudo/crates/rudo-gc-derive" }
rudo-gc-tokio-derive = { path = "learn-projects/rudo/crates/rudo-gc-tokio-derive" }
sys_alloc = { path = "learn-projects/rudo/crates/sys_alloc" }
libc = "0.2"
windows-sys = "0.52"
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "derive"] }
tempfile = "3.24"

# TO (crates.io):
[workspace.dependencies]
rudo-gc = { version = "0.8.13", features = ["test-util", "default", "tracing", "debug-suspicious-sweep", "tokio"] }
```

**Remove** these workspace deps when using crates.io (they are internal to rudo-gc and pulled in automatically):

- `rudo-gc-derive`
- `rudo-gc-tokio-derive`
- `sys_alloc`
- `libc`, `windows-sys`
- `proc-macro2`, `quote`, `syn`, `tempfile` (only needed by local rudo crates)

### 2. Update rvue crate (`crates/rvue/Cargo.toml`)

Remove the explicit `rudo-gc-derive` dependency (it comes via rudo-gc’s `derive` feature):

```toml
# FROM:
rudo-gc.workspace = true
rudo-gc-derive.workspace = true

# TO:
rudo-gc.workspace = true
```

### 3. Version check

Use the same rudo-gc version as on crates.io (e.g. `0.8.13`). Check [crates.io/rudo-gc](https://crates.io/crates/rudo-gc) for the latest.

---

## Switch TO local crate (crates.io → local)

Use this when you need to:

- Debug or modify rudo-gc
- Try unreleased changes
- Inspect rudo-gc internals

### 1. Ensure submodule is initialized

```bash
git submodule update --init learn-projects/rudo
```

### 2. Update workspace dependencies (`Cargo.toml`)

Replace the `[workspace.dependencies]` block:

```toml
# FROM (crates.io):
[workspace.dependencies]
rudo-gc = { version = "0.8.13", features = ["test-util", "default", "tracing", "debug-suspicious-sweep", "tokio"] }

# TO (local):
[workspace.dependencies]
rudo-gc = { path = "learn-projects/rudo/crates/rudo-gc", features = ["test-util", "default", "tracing", "debug-suspicious-sweep", "tokio"] }
rudo-gc-derive = { path = "learn-projects/rudo/crates/rudo-gc-derive" }
rudo-gc-tokio-derive = { path = "learn-projects/rudo/crates/rudo-gc-tokio-derive" }
sys_alloc = { path = "learn-projects/rudo/crates/sys_alloc" }
libc = "0.2"
windows-sys = "0.52"
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "derive"] }
tempfile = "3.24"
```

### 3. Update rvue crate (`crates/rvue/Cargo.toml`)

Add the explicit `rudo-gc-derive` dependency when using the local crate:

```toml
# FROM:
rudo-gc.workspace = true

# TO:
rudo-gc.workspace = true
rudo-gc-derive.workspace = true
```

### 4. Sync rudo submodule (optional)

To align with a specific rudo commit:

```bash
git submodule update learn-projects/rudo
```

---

## Feature flags (both sources)

Common feature set used in Rvue:

- `test-util` – `rudo_gc::test_util::reset()` (tests, examples)
- `default` – includes `derive`, `lazy-sweep`
- `tracing` – GC logging
- `debug-suspicious-sweep` – Vec<Gc<T>> misuse detection
- `tokio` – async / tokio integration

---

## Verification

After switching, run:

```bash
cargo build -p rvue -p rvue-macro -p rvue-style -p rvue-signals -p rvue-testing
cargo test -p rvue -- --test-threads=1
```

---

## Quick reference

| Direction      | rudo-gc source | rudo-gc-derive in rvue |
|----------------|----------------|-------------------------|
| crates.io      | `version = "X.Y.Z"` | Not needed (from derive feature) |
| local          | `path = "learn-projects/rudo/..."` | `rudo-gc-derive.workspace = true` |
