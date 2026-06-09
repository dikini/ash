# TASK-1380: Split `ash-typeck` expression/checking frontends

## Status: ✅ Complete

## Description

Split remaining oversized `ash-typeck` checking frontends, especially `check_expr.rs`, `lib.rs`, `check_pattern.rs`, `normalizer.rs`, and adjacent checker modules, after `TypeEnv` has a stable module layout.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- ✅ TASK-1379: Split `ash-typeck::type_env` into feature modules.

## Deferral / Planned-Feature Reconciliation

This task landed the behavior-preserving `check_expr` facade split and the first `lib.rs` surface-type-lowering extraction. `check_pattern.rs` and `normalizer.rs` remain oversized but were not split in this task because no equally safe semantic seam was needed to preserve the current task's verified checker-front-end boundary; they remain visible in the Phase 137 size report for final audit/follow-up ownership rather than being hidden behind unrelated churn.

## Requirements

### Functional Requirements

1. Split `check_expr.rs` by expression family or semantic responsibility.
2. Reduce `lib.rs` to public exports, orchestration, and stable entrypoints; move implementation details into named modules.
3. Split `check_pattern.rs` and `normalizer.rs` only along existing semantic seams.
4. Preserve diagnostic variants and public typechecker entrypoints.
5. Keep existing tests green and add no semantic changes.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Identify call graph boundaries

Search for public functions in `check_expr.rs`, `lib.rs`, `check_pattern.rs`, and `normalizer.rs`; classify what must remain public.

### Step 2: Extract expression families incrementally

Suggested candidates: literals/variables, calls, pattern forms, workflow forms, law/proof checks if they are currently embedded in large files.

### Step 3: Keep diagnostics stable

Run focused diagnostics suites after every extraction:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck diagnostics -- --nocapture
```

### Step 4: Run broad `ash-typeck` gates

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
```

### Step 5: Codex review

Ask Codex to inspect for accidental behavior changes, duplicated logic, import churn, and diagnostic masking.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1380-size.md
checklist:
  - [x] Refactor is behavior-preserving
  - [x] Public API paths preserved or deliberately documented
  - [x] ash-typeck tests pass
  - [x] ash-typeck clippy is clean
  - [x] Formatting and diff checks pass
  - [x] Size report shows intended reduction and explicitly documents remaining over-ceiling checker/root outliers for final audit ownership
  - [x] Codex final review reports no blocking issues
```


## Remaining Size Ownership

The Phase 137 size report still lists `crates/ash-typeck/src/check_expr/mod.rs` and `crates/ash-typeck/src/lib.rs` above the per-task 2,500-line target after this safe extraction slice. They are not treated as invisible debt: TASK-1387's closeout audit must either split them further or record explicit follow-up owners alongside the still-oversized `check_pattern.rs` and `normalizer.rs`. TASK-1380 is limited to the behavior-preserving `check_expr` feature-module extraction plus `surface_type_lowering` helper extraction verified by the crate-local gates.

## Dependencies for Next Task

This task outputs:
- `ash-typeck::check_expr` is organized as a feature-owned module directory with a compatibility facade plus `result`, `pattern_bridge`, `core`, and `do_notation` slices.
- `lib.rs` has begun moving implementation details into named helper modules via `surface_type_lowering.rs`, while preserving existing crate-root compatibility paths such as `crate::bind_pattern_variables`.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.


## Completion Notes

- Converted `crates/ash-typeck/src/check_expr.rs` into `crates/ash-typeck/src/check_expr/mod.rs` plus feature/helper modules (`result`, `pattern_bridge`, `core`, `do_notation`) while preserving `ash_typeck::check_expr::*` public entrypoints and crate-visible bridge helpers.
- Extracted `surface_type_lowering.rs` from `lib.rs` and preserved existing `crate::bind_pattern_variables` compatibility for sibling modules.
- Verification before Codex review: `cargo fmt --check`, `git diff --check`, `cargo test -p ash-typeck` (607 unit tests plus doctests), `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, and `tools/dev/rust_file_size_report.py --fail-on-regression` all exited 0 on the task diff.
- Remaining over-ceiling files after this slice are explicitly owned by the Phase 137 final audit/follow-up surface rather than hidden: `check_expr/mod.rs` remains a compatibility facade around the moved feature modules, `lib.rs` still contains public orchestration/root glue after the `surface_type_lowering` extraction, and `check_pattern.rs`/`normalizer.rs` remain visible oversized follow-up candidates. This task intentionally avoided speculative extraction without a safe semantic seam.
- Codex re-review after remediation reported no discrete correctness issues in the current staged, unstaged, or untracked changes.
