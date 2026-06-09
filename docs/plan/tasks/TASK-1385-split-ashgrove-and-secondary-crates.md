# TASK-1385: Split ashgrove and secondary oversized crates

## Status: ✅ Complete

## Description

Split `ashgrove/src/lib.rs` and remaining secondary oversized crates (`ash-provenance`, `ash-repl`, `ash-lint`, `ash-lsp`, `ash-lsp-core`, `ash-mcp`) so their public entry files become navigational rather than implementation-heavy.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Split `ashgrove/src/lib.rs` into command, manifest, install, update, remove/cleanup, trust, source-payload, vendor, and test-support-facing modules as supported by current code.
2. Split secondary crate root files that exceed 500 lines by feature.
3. Preserve CLI/library public APIs and test fixture behavior.
4. Avoid cross-crate helper extraction unless it is already an accepted API.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Split `ashgrove` first

Run:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ashgrove
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
```

### Step 2: Split secondary crates in descending size order

For each crate, run its focused tests/checks before moving to the next crate.

### Step 3: Codex review

Ask Codex to verify CLI behavior, fail-closed path/trust semantics, and public API compatibility.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ashgrove
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-provenance
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-repl
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-lint
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-lsp-core
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-mcp
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ashgrove -p ash-provenance -p ash-repl -p ash-lint -p ash-lsp -p ash-lsp-core -p ash-mcp --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1385-size.md
checklist:
  - [x] Refactor is behavior-preserving
  - [x] Public API paths preserved or deliberately documented
  - [x] Listed crate tests pass
  - [x] Listed crate clippy is clean
  - [x] Formatting and diff checks pass
  - [x] Size report shows intended reduction or documented exception
  - [x] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- `ashgrove` and secondary crates have smaller root modules.
- No secondary crate root remains oversized without documented reason.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
