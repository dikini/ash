# TASK-1381: Split parser surface/lowering/import resolver modules

## Status: ✅ Complete

## Description

Split the largest parser files (`surface.rs`, `parse_module.rs`, `lower.rs`, `import_resolver.rs`, `parse_expr.rs`, `parse_workflow.rs`, and `lift.rs`) into feature-owned modules while preserving parser/lowering behavior and AST compatibility.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

This task was completed as a behavior-preserving module-size split. It intentionally did not redesign parser dispatch, AST ownership, import visibility semantics, lowering behavior, or public parser APIs. Remaining parser production outliers are explicitly narrowed and owned by TASK-1387/future size work:

- `crates/ash-parser/src/parse_module.rs` remains 3,103 lines after extracting function-definition parsing and tests; the residual file is still the central module-item dispatch and mixed definition parser surface.
- `crates/ash-parser/src/surface.rs` remains 2,459 lines after moving test/proptest modules; the residual file is the compatibility AST surface.
- `crates/ash-parser/src/lower.rs` remains 2,443 lines after moving lowering tests; the residual file is the compatibility lowering facade and semantic lowering helpers.

These residuals are documented exceptions for this task rather than hidden failures; TASK-1387 owns final audit/status reconciliation and any follow-on split owners.

## Requirements

### Functional Requirements

1. Split `surface.rs` into AST groups: module items, interfaces/impls, expressions/workflows, tests/helpers where appropriate.
2. Split `parse_module.rs` by module item parser family while keeping dispatch explicit.
3. Split `lower.rs`/`lift.rs` by source/target semantic families.
4. Split `import_resolver.rs` by graph discovery, module loading, summary merging, and diagnostics if live seams support that.
5. Keep parser keyword and dispatch behavior unchanged.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Create parser module map

Completed. The verified parser split map is:

- `import_resolver.rs` is now a production facade with binding/error carriers in `import_resolver/types.rs` and unit tests in `import_resolver/tests.rs`.
- `parse_module.rs` keeps central module-item dispatch and compatibility entrypoints, with function/builtin-function parsing extracted to `parse_module/fn_defs.rs` and tests in `parse_module/tests.rs`.
- `surface.rs` keeps the compatibility AST surface, with large test/proptest modules moved to `surface/tests.rs`, `surface/effect_tests.rs`, and `surface/visibility_tests.rs`.
- `lower.rs`, `lift.rs`, `parse_expr.rs`, and `parse_workflow.rs` keep production compatibility facades while moving their tail test modules to sibling `tests.rs` files.

### Step 2: Extract surface AST groups with reexports

Completed as a conservative compatibility split. Public AST definitions stayed in `surface.rs`; large surface test modules moved out. This avoids churn in the heavily used `ash_parser::surface::*` API while still reducing production context load.

### Step 3: Extract parser dispatch helpers carefully

Completed for the safe contiguous function-definition parser family in `parse_module/fn_defs.rs`, preserving public `parse_fn_definition`, `parse_builtin_fn_definition`, and `parse_fn_body` reexports. After each extraction, focused/full parser tests were run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser
```

### Step 4: Validate downstream crates

Because parser types cross into engine/typeck/LSP, run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
```

### Step 5: Codex review

Ask Codex to verify no surface AST fields or lowering metadata were dropped during extraction.


## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
  - git diff --check
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1381-size.md
checklist:
  - [x] Refactor is behavior-preserving
  - [x] Public API paths preserved or deliberately documented
  - [x] ash-parser tests pass
  - [x] workspace check passes for downstream parser users
  - [x] ash-parser clippy is clean
  - [x] Formatting and diff checks pass
  - [x] Size report shows intended reduction or documented exception
  - [x] Codex final review reports no blocking issues
```

### Verification Evidence

Commands run with `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER=` where applicable:

- `cargo fmt --check` plus `git diff --check` passed after final formatting.
- `cargo test -p ash-parser` passed, including parser unit tests, integration tests, and parser doctests.
- `cargo check --workspace` passed for downstream parser users.
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings` passed.
- `python3 tools/dev/rust_file_size_report.py --fail-on-regression` passed with no Phase 137 baseline regressions.
- Size spot-check after this task: `import_resolver.rs` 775 lines, `parse_module.rs` 3,103 lines, `parse_module/fn_defs.rs` 688 lines, `lower.rs` 2,443 lines, `surface.rs` 2,459 lines, `parse_expr.rs` 1,782 lines, `parse_workflow.rs` 1,851 lines, `lift.rs` 749 lines.


## Dependencies for Next Task

This task outputs:
- Parser modules split by semantic family.
- `surface.rs` and lowering files no longer require loading the entire parser surface for small edits.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
