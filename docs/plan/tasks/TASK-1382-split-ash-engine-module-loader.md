# TASK-1382: Split engine module loading and public engine shell

## Status: ✅ Complete

## Description

Split `ash-engine` mega-files, prioritizing `module_loader.rs` and `lib.rs`, into loader phases and public engine API modules without changing import, check, run, or RuntimeKernel behavior.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Split `module_loader.rs` into discovery, parsing/checking, semantic-summary import/export, stdlib/dependency resolution, and diagnostics modules where supported by the current code.
2. Reduce `src/lib.rs` to API surface, builders, and reexports; move implementation-heavy logic into named modules.
3. Keep `Engine`, `EngineBuilder`, and public helper paths stable.
4. Run module resolution and stdlib corpus tests after each major split.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Map `module_loader.rs` phases

Identify natural phase functions and data carriers before extracting modules.

### Step 2: Extract one loader phase at a time

After each move, run focused tests:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine module_resolution -- --nocapture
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine module_file_check_tests -- --nocapture
```

### Step 3: Verify public API stability

Run workspace check to catch downstream import changes.

### Step 4: Codex review

Ask Codex to focus on import resolution, semantic summaries, public API compatibility, and RuntimeKernel side effects.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1382-size.md
checklist:
  - [x] Refactor is behavior-preserving
  - [x] Public API paths preserved or deliberately documented
  - [x] ash-engine tests pass
  - [x] workspace check passes for downstream engine users
  - [x] ash-engine clippy is clean
  - [x] Formatting and diff checks pass
  - [x] Size report shows intended reduction or documented exception
  - [x] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- `ash-engine` loader/API modules split by runtime responsibility.
- `module_loader.rs` no longer dominates engine changes.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
## Implementation Evidence

- Extracted `module_loader/source_scan.rs` for source scanning helpers.
- Extracted `module_loader/import_resolution.rs` for dependency/stdlib/module-root resolution and lockfile identity checks.
- Extracted `module_loader/callable_exports.rs` for public capability, type-snippet, workflow, builtin callable, and public workflow-summary export helpers.
- Moved inline `module_loader` unit tests to `module_loader/tests.rs` and public engine-shell unit tests to `tests.rs`.
- Preserved public seams including `Engine`, `EngineBuilder`, `with_module_roots`, module check/load helpers, and existing compatibility call paths.

Verified:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
git diff --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo check --workspace
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```

Post-split line counts:

```text
2640 crates/ash-engine/src/lib.rs
 777 crates/ash-engine/src/tests.rs
5409 crates/ash-engine/src/module_loader.rs
 693 crates/ash-engine/src/module_loader/callable_exports.rs
 714 crates/ash-engine/src/module_loader/import_resolution.rs
 194 crates/ash-engine/src/module_loader/source_scan.rs
1248 crates/ash-engine/src/module_loader/tests.rs
```

Remaining size exceptions / follow-on ownership:

- `crates/ash-engine/src/module_loader.rs` remains a 5,409-line compatibility/orchestration shell because the next safe reductions require disentangling semantic-summary import/export registration from module-file checking; TASK-1387 must carry this as a Phase 138 follow-on candidate rather than forcing a risky semantic-summary move inside this behavior-preserving task.
- `crates/ash-engine/src/lib.rs` remains a 2,640-line public API and builder shell because the remaining heavy logic is intertwined with public `Engine`/`EngineBuilder` docs, doctests, provider registration, and compatibility helpers; TASK-1387 must record this as a follow-on public-shell extraction candidate unless a later Phase 137 task safely removes it from the >2,500-line budget.
- These exceptions preserve behavior and API stability while still reducing `module_loader.rs` from the Phase 137 baseline 8,248 lines and splitting public-shell tests out of `lib.rs`.
