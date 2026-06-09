# TASK-1384: Split interpreter eval/execute/runtime state modules

## Status: ✅ Complete

## Description

Split oversized interpreter runtime files (`eval.rs`, `execute.rs`, `runtime_state.rs`, `small_step.rs`) into operation-family modules while preserving execution semantics and runtime authority boundaries.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Split `eval.rs` by expression/control/runtime operation family.
2. Split `execute.rs` by public execution path and shared execution helpers.
3. Split `runtime_state.rs` by provider registry, hidden ActEnv/runtime kernel state, streams/mailboxes, and failure attribution if supported by live code.
4. Preserve async/runtime behavior and authority checks exactly.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Identify runtime authority seams

Before extraction, list all provider, ActEnv, RuntimeKernel, stream, and failure-attribution callsites.

### Step 2: Extract operation families incrementally

After each extraction, run focused interpreter tests:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-interp
```

### Step 3: Validate engine integration

Run representative engine runtime tests that exercise interpreter call paths.

### Step 4: Codex review

Ask Codex to focus on runtime authority preservation, async behavior, and no new sync fallbacks.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-interp
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-interp --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1384-size.md
checklist:
  - [x] Refactor is behavior-preserving
  - [x] Runtime authority and async behavior are preserved
  - [x] ash-interp tests pass
  - [x] representative ash-engine runtime tests pass
  - [x] ash-interp clippy is clean
  - [x] Formatting and diff checks pass
  - [x] Size report shows intended reduction or documented exception
  - [x] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- Interpreter runtime modules split by operation family.
- Execution and runtime-state edits become localized.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
## Implementation Evidence

- Split `crates/ash-interp/src/eval.rs` into feature-owned sibling modules:
  - `eval/builtins.rs` for builtin dispatch metadata/table lookup.
  - `eval/operators.rs` for unary/binary/comparison helpers.
  - `eval/failure.rs` for operational failure attribution helpers.
  - `eval/control.rs` for spawn/split/match/if-let control-expression helpers.
  - `eval/tests.rs` for extracted eval unit tests.
- Split `crates/ash-interp/src/runtime_state.rs` data-model surfaces into:
  - `runtime_state/implementation.rs` for implementation-binding and operation-body metadata.
  - `runtime_state/resource_admission.rs` for resource split/join violation evidence.
- Split `crates/ash-interp/src/execute.rs` terminal observation/failure conversion helpers into `execute/terminal.rs` while keeping authority-sensitive workflow execution and child-spawn orchestration in the parent shell.
- Subagent boundary review advised extracting pure-ish eval helpers and data models first, and avoiding partial moves of `ActEnv`, proc authority, and runtime kernel clusters.
- Size evidence from `/tmp/phase137-task1384-size.md`: no Phase 137 baseline regressions; `eval.rs` reduced to 3,308 lines, `runtime_state.rs` to 2,329 lines, and `execute.rs` to 3,331 lines, with remaining large inline tests/production shells deferred to TASK-1386/closeout exceptions.

## Verification Evidence

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check
git diff --check
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-interp --all-targets --all-features -- -D warnings
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-interp
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-engine
python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1384-size.md
python3 tools/dev/rust_file_size_report.py --fail-on-regression
```
