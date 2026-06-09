# TASK-1386: Split oversized test/support files by behavior

## Status: 📝 Planned

## Description

Split oversized integration test and support files after production modules have stabilized, preserving test names, coverage, and non-zero verification evidence.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1379 through TASK-1385 production splits should be complete or explicitly deferred before broad test fixture splits.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Identify test files above 500 lines and above 10KB using the Phase 137 audit script.
2. Split tests by behavior area, historical task group, or fixture family.
3. Preserve existing test names where possible so prior evidence remains searchable.
4. Move shared helpers to local `support` modules rather than copy-pasting fixtures.
5. Ensure no split causes zero-test filtered commands in task docs.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Generate oversized test list

```bash
python3 tools/dev/rust_file_size_report.py --markdown --tests-only > /tmp/oversized-tests.md
```

### Step 2: Split high-impact test files first

Prioritize test files in `ash-typeck`, `ash-engine`, `ash-cli`, and `ashgrove` that currently exceed 500 lines.

### Step 3: Verify exact test binaries

For each split test file, run the exact old/new test target and confirm non-zero test counts.

### Step 4: Codex review

Ask Codex to verify no tests were dropped, renamed misleadingly, or converted into fixture-only coverage.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown --tests-only > /tmp/phase137-task1386-tests-size.md
checklist:
  - [ ] Test refactor is behavior-preserving
  - [ ] Existing test intent/names preserved or documented
  - [ ] Workspace tests pass with non-zero affected test counts
  - [ ] Workspace clippy is clean
  - [ ] Formatting and diff checks pass
  - [ ] Size report shows intended reduction or documented exception
  - [ ] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- Oversized tests split into behavior-focused files.
- Shared test helpers extracted without weakening coverage.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
