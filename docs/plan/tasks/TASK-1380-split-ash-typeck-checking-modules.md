# TASK-1380: Split `ash-typeck` expression/checking frontends

## Status: 📝 Planned

## Description

Split remaining oversized `ash-typeck` checking frontends, especially `check_expr.rs`, `lib.rs`, `check_pattern.rs`, `normalizer.rs`, and adjacent checker modules, after `TypeEnv` has a stable module layout.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1379: Split `ash-typeck::type_env` into feature modules.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

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
  - [ ] Refactor is behavior-preserving
  - [ ] Public API paths preserved or deliberately documented
  - [ ] ash-typeck tests pass
  - [ ] ash-typeck clippy is clean
  - [ ] Formatting and diff checks pass
  - [ ] Size report shows intended reduction or documented exception
  - [ ] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- `ash-typeck` checker frontends are organized by feature.
- `lib.rs` is a navigation surface rather than a large implementation file.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
