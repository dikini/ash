# TASK-1379: Split `ash-typeck::type_env` into feature modules

## Status: 📝 Planned

## Description

Split the 20,935-line `crates/ash-typeck/src/type_env.rs` into discoverable feature modules while preserving the `TypeEnv` API and all existing semantics.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Convert `crates/ash-typeck/src/type_env.rs` into a thin module shell or `type_env/mod.rs` root.
2. Extract cohesive areas into child modules such as `builtins`, `interfaces`, `evidence`, `summaries`, `associated_families`, `constructors`, `proofs`, and `tests` where the live code supports those boundaries.
3. Preserve existing public imports from `ash_typeck::type_env::*` using `pub use` as needed.
4. Keep helper APIs crate-private unless already public.
5. Update intra-crate imports mechanically without changing behavior.
6. Split or relocate embedded tests only when that reduces context size without weakening coverage.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Survey internal sections

Use symbol/heading search over `crates/ash-typeck/src/type_env.rs` to identify natural feature clusters before moving code.

### Step 2: Add module shell and one extraction at a time

Move one cohesive cluster, run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck type_env -- --nocapture
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check -p ash-typeck
```

### Step 3: Preserve API compatibility

Run searches for `type_env::` and `TypeEnv` callsites across the workspace and update only imports that must change.

### Step 4: Full crate gate

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-typeck
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
```

### Step 5: Size audit

Run the Phase 137 size script and record before/after for `ash-typeck`.

### Step 6: Codex review

Ask Codex to verify that the split is mechanical, public APIs are preserved, and new module visibility is not over-widened.


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
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1379-size.md
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
- `type_env` split into feature-owned modules.
- Largest `ash-typeck` file size materially reduced.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
