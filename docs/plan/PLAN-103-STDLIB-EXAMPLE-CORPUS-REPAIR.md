# PLAN-103: Stdlib and Example Corpus Repair

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 107 is a remediation phase: lock the broken `ash check` corpus baseline first, then fix std module/import issues before rewriting examples. Do not broaden parser syntax just to preserve historical sketches unless a task explicitly says so.

**Goal:** Make the Ash standard library and intended checkable examples align with the current parser/typechecker/module-loader surface after Phases 105 and 106.

**Architecture:** Phase 107 is a corpus-conformance and module-resolution repair phase. It starts with a reproducible CLI `ash check` corpus harness, fixes real std/module-loader defects exposed by `std/src/**/*.ash`, then canonicalizes or explicitly labels older examples that use historical/reference syntax. The phase distinguishes executable conformance examples from non-checkable design sketches rather than silently treating every historical file as current syntax.

**Tech Stack:** `ash-cli check`, `ash-engine` module loader/import resolution, `ash-parser` comment and diagnostic surfaces, std `.ash` modules, example `.ash` corpus, Rust 2024 tests.

---

## Phase 107: Stdlib and Example Corpus Repair

**Status:** 📝 Planned
**Depends on:** Phase 105 generalized typed do notation, Phase 106 monad comprehension syntax, existing std module loader.
**Investigation baseline:** 2026-04-28 on `34f083f`.

### Baseline Findings

Fresh `ash check` sweeps in the Phase 107 worktree found:

| Corpus | Files | Passing | Failing |
|--------|-------|---------|---------|
| `std/src/**/*.ash` | 39 | 33 | 6 |
| `examples/**/*.ash` | 36 | 19 | 17 |

Passing modern syntax examples include all Phase 105 and Phase 106 examples. The broken corpus is not a Phase 105/106 regression by itself; failures cluster around std module/import resolution and older example syntax.

### Stdlib Failure Buckets

1. **Multiline import pre-scan bug**: `std/src/llm/dispatch.ash` has `use types::{ ... };`, but `ash-engine` scans imports line-by-line and tries to parse only `use types::{`.
2. **Module-root re-export handling**: `std/src/io/mod.ash` is a `pub mod` / `pub use` root and currently fails through the CLI `ash check` path.
3. **Workflow export visibility/importability**: `std/src/llm/{conversation,router,supervised,tool_agent}.ash` fail to import `dispatch::complete` or `dispatch::complete_with_tools`.
4. **Relative import resolution**: `std/src/runtime/supervisor.ash` uses `super::error` / `super::args`, but module resolution treats `super` literally.
5. **Incorrect nested std imports**: `std/src/llm/loading.ash` imports `path` where the real module is under `io/path.ash` (and similarly `fs`).

### Example Failure Buckets

1. **`runtime::Args` re-export/import drift**: `examples/entrypoint_args.ash` cannot resolve `runtime::Args` even though std files claim to export it.
2. **Line comment mismatch**: parser docs mention `//`, but comment skipping accepts only `--` and `/* ... */`; many older examples begin with `//`.
3. **Historical workflow syntax**: older examples use forms not accepted by current parser, including `if cond { ... }`, `for x in xs { ... }`, `decide ... else`, `observe ... with`, `act ... with`, role-shaped `with`, and obligation/policy sketches.
4. **Opaque parse diagnostics**: most example failures collapse to raw `ContextError`, hiding the actionable grammar mismatch.

### Task Table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-760](tasks/TASK-760-cli-corpus-baseline-harness.md) | Add std/example `ash check` corpus harness and baseline classification | Test/Scaffold | 4 | ✅ Complete |
| [TASK-761](tasks/TASK-761-stdlib-multiline-imports-and-module-roots.md) | Fix multiline imports and module-root re-export checking | Substrate | 6 | ✅ Complete |
| [TASK-762](tasks/TASK-762-stdlib-workflow-export-and-relative-imports.md) | Fix workflow export visibility plus relative/super imports | Substrate | 6 | ✅ Complete |
| [TASK-763](tasks/TASK-763-runtime-args-and-llm-loading-imports.md) | Repair `runtime::Args` and `llm/loading.ash` std import surfaces | Semantic | 5 | ✅ Complete |
| [TASK-764](tasks/TASK-764-parser-comments-and-diagnostics.md) | Add `//` comment support and targeted parse diagnostics for common stale syntax | Parser/DX | 6 | 📝 Planned |
| [TASK-765](tasks/TASK-765-canonicalize-small-examples.md) | Canonicalize small control-flow and IO examples to current syntax | Examples | 6 | 📝 Planned |
| [TASK-766](tasks/TASK-766-reference-example-policy-and-closeout.md) | Decide/canonicalize/mark large reference examples and close Phase 107 | Docs/Examples | 6 | 📝 Planned |

Estimated total: 39 hours.

## Execution Order

This phase is intentionally sequential:

1. TASK-760 must run first to lock a measurable corpus baseline.
2. TASK-761 and TASK-762 unblock std modules and must precede most std/example import repairs.
3. TASK-763 depends on the std import substrate from TASK-761/762.
4. TASK-764 can run after TASK-760 but should land before large example rewrites so parse failures become diagnosable.
5. TASK-765 handles small examples after comment/diagnostic support exists.
6. TASK-766 decides the fate of large historical/reference examples and performs final corpus closeout.

## Implementation Constraints

1. Do not change Phase 105/106 syntax semantics while fixing corpus drift.
2. Do not accept all historical example syntax blindly; either canonicalize to current Ash or label/move as reference-only.
3. Keep corpus tests honest: avoid a green test that only exercises `Engine::check_module_file` when `ash-cli check` still fails.
4. Prefer small, targeted module-loader fixes over global parser relaxations.
5. If a file is intentionally non-checkable, mark that policy explicitly and exclude it through a documented corpus rule.
6. Update `CHANGELOG.md` and `PLAN-INDEX.md` with every task completion.

## Verification Strategy

Per task:

- Run targeted `cargo test` for new regression tests.
- Run `cargo run -q -p ash-cli -- check <affected .ash files>`.
- Run `cargo fmt --check` and `git diff --check`.
- Run affected-crate `cargo check` / `cargo clippy --all-targets --all-features -- -D warnings` where Rust code changes.
- Use independent review before marking the task complete.

Phase closeout:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo doc --workspace --no-deps`
- final std/example corpus report with exact pass/fail counts
- PLAN-INDEX / CHANGELOG / task status reconciliation
