# TASK-359 Stdlib Initialization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend `ash-std` with the Phase 57 entry-point foundation by adding the `runtime` stdlib module tree, canonical `RuntimeError` and `Args` declarations, and a supervisor module scaffold without prematurely implementing runtime bootstrap behavior.

**Architecture:** Add stdlib-facing Ash source files under `std/src/runtime/`, expose them through `runtime/mod.ash` and `std/src/lib.ash`, and prove the new surface with focused stdlib parser tests. Keep the work textual and module-structure-oriented so downstream runtime tasks can build on a stable importable stdlib contract.

**Tech Stack:** Rust workspace tests in `crates/ash-parser/tests`, Ash stdlib source files in `std/src`, and changelog bookkeeping in `CHANGELOG.md`.

---

## Planning assumptions

- `TASK-359` is a foundation task only; it must not absorb runtime/bootstrap implementation from later blocked tasks.
- The normative surface is determined by completed S57-4, S57-5, and S57-6 spec updates.
- Focused stdlib tests are sufficient; this task does not need a new broad compile harness unless existing tests prove insufficient.
- No git commit should be performed in this session unless the user asks for it.

## Verification defaults

- `cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet`
- `cargo build -p ash-std`
- `git diff --check`

---

### Task 1: Add failing stdlib tests for the runtime module tree

**Files:**

- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`

**Step 1: Write failing existence tests**

Add tests asserting these files exist under `std/src/runtime/`:

- `mod.ash`
- `error.ash`
- `args.ash`
- `supervisor.ash`

**Step 2: Write failing surface-content tests**

Add focused assertions that:

- `runtime/error.ash` contains `pub type RuntimeError`
- `runtime/args.ash` contains `pub capability Args`
- `runtime/mod.ash` re-exports the runtime symbols
- `std/src/lib.ash` exposes the runtime module surface

**Step 3: Run red tests**

Run:

```bash
cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet
```

Expected: failures because the runtime stdlib files and exports do not exist yet.

**Step 4: Stop after confirming red**

Do not implement production changes until the failures are observed.

---

### Task 2: Create the runtime stdlib module tree

**Files:**

- Create: `std/src/runtime/mod.ash`
- Create: `std/src/runtime/error.ash`
- Create: `std/src/runtime/args.ash`
- Create: `std/src/runtime/supervisor.ash`

**Step 1: Add `RuntimeError`**

Create `std/src/runtime/error.ash` with the canonical public record-style ADT:

```ash
pub type RuntimeError = RuntimeError {
    exit_code: Int,
    message: String
};
```

**Step 2: Add `Args`**

Create `std/src/runtime/args.ash` using declaration-site capability syntax and `Option<String>` as the return type.

**Step 3: Add supervisor scaffold**

Create `std/src/runtime/supervisor.ash` with the downstream-facing public symbol `system_supervisor(args: cap Args) -> Int`, but keep only a placeholder body so `TASK-359` does not absorb the runtime/bootstrap behavior reserved for `TASK-362` and `TASK-363c`.

**Step 4: Add runtime root module**

Create `std/src/runtime/mod.ash` that re-exports the runtime symbols from the sibling modules.

---

### Task 3: Export the runtime surface from the stdlib root

**Files:**

- Modify: `std/src/lib.ash`

**Step 1: Extend `lib.ash` minimally**

Add runtime re-exports in the same style as the existing Option/Result surface.

**Step 2: Preserve current exports**

Do not reorder or rewrite unrelated Option/Result exports beyond what is necessary to add runtime exports.

---

### Task 4: Verify the new stdlib surface goes green

**Files:**

- Review only; no new files expected unless a test reveals a missing export

**Step 1: Run focused tests**

Run:

```bash
cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet
```

Expected: all stdlib parsing/surface tests pass.

**Step 2: Run the focused stdlib build**

Run:

```bash
cargo build -p ash-std
```

Expected: success.

**Step 3: Fix only test-proven gaps**

If failures remain, make the smallest production change needed to satisfy them.

---

### Task 5: Update changelog and final verification

**Files:**

- Modify: `CHANGELOG.md`

**Step 1: Add unreleased entry**

Add a concise `### Added` or `### Changed` entry for `TASK-359` describing the new runtime stdlib surface.

**Step 2: Run final verification**

Run:

```bash
cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet
cargo build -p ash-std
git diff --check
```

Expected: all commands succeed.

**Step 3: Prepare review handoff**

Summarize touched files, results, and any follow-up constraints for downstream tasks (`360`-`367`).

---

## Execution mode for this session

Use subagent-driven development in this worktree:

1. implementation subagent for failing tests and code changes;
2. spec-compliance review subagent;
3. code-quality review subagent;
4. final verification before completion.
