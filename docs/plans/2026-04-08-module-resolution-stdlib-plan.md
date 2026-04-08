# General Module Resolution And Stdlib-Backed Workflows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `ash run` execute ordinary workflow files that import stdlib modules and user-defined modules through a real module resolver.

**Architecture:** Move import resolution into `ash-engine` so file execution uses one graph-backed loader regardless of whether the workflow is ordinary or canonical entry-shaped. Resolve modules from the workflow file's directory tree first, then `ASH_LIBRARY_PATH` in order, then the built-in stdlib root. Preserve canonical entry semantics on top of the same loader. Enforce one concrete version per library name when imports use version-qualified library syntax.

**Tech Stack:** Rust workspace (`ash-engine`, `ash-cli`), filesystem-backed module loading, environment-variable root discovery, repository tests

## Assumptions

- The existing parser can continue to parse workflow/module source once the resolver feeds it the resolved source text and imported definitions.
- The current narrow entry bootstrap path remains available, but import loading moves to the shared resolver.
- `ASH_LIBRARY_PATH` is a PATH-like list of directories, parsed with the platform path separator.

## Non-Goals

- package installation
- manifest-driven dependency solving
- remote module fetching
- arbitrary version ranges beyond the explicit version-qualified bootstrap syntax
- redefining the stdlib surface itself

---

### Task 1: Add failing resolver tests

**Files:**
- Create or modify: `crates/ash-engine/tests/module_resolution.rs`
- Create or modify: `crates/ash-cli/tests/run_output.rs`

**Step 1: Write failing engine tests for local-tree resolution**

Cover resolving imports from files in the workflow tree, including nested multi-file modules.

**Step 2: Write failing engine tests for stdlib-backed ordinary workflows**

Cover ordinary files importing stdlib modules such as `option`, `prelude`, and `std/lib`.

**Step 3: Write failing engine tests for `ASH_LIBRARY_PATH`**

Cover search ordering across multiple library directories and precedence versus the local tree and stdlib root.

**Step 4: Write failing engine tests for version-qualified imports**

Cover `use math@1::vector` style imports, single-version enforcement, and version conflicts.

**Step 5: Write failing engine tests for unqualified external-library imports**

Cover unqualified external-library imports being rejected as ambiguous unless future packaging/manifest work defines a disambiguation rule.

**Step 6: Write failing CLI tests for `ash run`**

Cover file-backed workflows that execute successfully once imports are resolved through the graph loader.

**Likely files to touch later**

- `crates/ash-engine/tests/module_resolution.rs`
- `crates/ash-cli/tests/run_output.rs`
- `crates/ash-engine/src/lib.rs`
- `crates/ash-engine/src/module_resolver.rs`
- `crates/ash-engine/src/entry.rs`
- `crates/ash-cli/src/commands/run.rs`

### Task 2: Implement the engine resolver

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Create: `crates/ash-engine/src/module_resolver.rs`
- Modify: `crates/ash-engine/src/entry.rs`
- Modify: `crates/ash-engine/src/error.rs`
- Modify: `crates/ash-engine/src/parse.rs`
- Modify: `crates/ash-engine/src/check.rs`

**Step 1: Add resolver configuration**

Model:
- root workflow file path
- local root
- `ASH_LIBRARY_PATH` directories
- built-in stdlib root
- selected version per library name

**Step 2: Implement filesystem resolution**

Resolve module paths against candidate files and directories, with deterministic precedence and cycle detection.

**Step 3: Load stdlib modules through the same resolver**

Expose stdlib modules as ordinary resolver roots so imported stdlib symbols are available in non-entry files.

**Step 4: Enforce version-qualified library selection**

Reject mixed requests for different versions of the same library name within one graph.

**Likely helpers to add**

- `resolve_module_path`
- `resolve_library_root`
- `load_module_graph`
- `load_library_metadata`
- `parse_ash_library_path`

### Task 3: Thread the resolved graph into execution

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/execute.rs`
- Modify: `crates/ash-engine/src/check.rs`
- Modify: `crates/ash-engine/src/entry.rs`

**Step 1: Replace single-source file execution with graph-backed execution**

Make file-backed workflows execute after module loading and import resolution.

**Step 2: Keep entry bootstrap semantics on the same loader**

Canonical entry workflows should still work, but they should be fed by the same resolver-backed graph.

**Step 3: Surface clear resolution errors**

Return explicit errors for missing modules, import cycles, and version conflicts.

**Likely entrypoints to update**

- `Engine::execute`
- `Engine::check`
- `Engine::run`
- `Engine::run_file`
- `Engine::parse_entry_source`
- `Engine::bootstrap_entry_source`

### Task 4: Integrate the CLI

**Files:**
- Modify: `crates/ash-cli/src/commands/run.rs`
- Modify: `crates/ash-cli/tests/run_output.rs`

**Step 1: Route `ash run <file>` through the graph-backed loader**

Remove the current dependency on narrow source-classification for import handling.

**Step 2: Keep ordinary execution and entry bootstrap behavior correct**

Preserve current output and exit-code semantics while imports are now resolved through the engine graph.

**Step 3: Add `ASH_LIBRARY_PATH` integration coverage**

Verify the CLI honors the search order and version conflict behavior.

### Task 5: Verify behavior end to end

**Step 1: Run engine tests**

Verify local tree, stdlib, versioning, and cycle resolution cases.

**Step 2: Run CLI tests**

Verify ordinary file execution with imports and entry workflows still work.

**Step 3: Run quality checks**

Run focused `cargo test` and `cargo clippy` for the affected crates before declaring the work done.

## Verification Commands

Run these commands at the end of implementation:

```bash
cargo test -p ash-engine
cargo test -p ash-cli
cargo fmt --check
cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings
```
