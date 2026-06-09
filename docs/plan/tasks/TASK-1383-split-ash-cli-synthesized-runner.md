# TASK-1383: Split synthesized test runner modules

## Status: 📝 Planned

## Description

Split `crates/ash-cli/src/test_runner/synthesized.rs` and adjacent runner files by synthesized-row family, improving LLM load size and discoverability for contract, policy, obligation, small-world, and law-test logic.

## Specification Reference

- [PLAN-137: Rust Module Size and Discoverability Refactor](../PLAN-137-RUST-MODULE-SIZE-REFACTOR.md)
- Rust split rules from `rust-skills`: `proj-mod-by-feature`, `proj-pub-crate-internal`, `proj-pub-use-reexport`, `own-borrow-over-clone`, `anti-over-abstraction`, `lint-rustfmt-check`.

## Dependencies

- 📝 TASK-1378: Add module-size audit and split policy.

## Deferral / Planned-Feature Reconciliation

None. This is a behavior-preserving refactor task; any discovered semantic redesign belongs in a separate future phase.

## Requirements

### Functional Requirements

1. Split synthesized runner logic into modules such as `contract`, `policy`, `obligation`, `smallworld`, `law`, `metadata`, and `repro` as supported by current code.
2. Keep `test_runner` public/internal API stable for executor and CLI command users.
3. Preserve filtering/fail-fast semantics exactly.
4. Keep JSON/human output shape stable.

### Size / Discoverability Requirements

1. Prefer feature-owned modules below 500 lines.
2. Preserve stable public API paths with compatibility reexports where existing callsites require them.
3. Keep helper modules `pub(crate)` / `pub(super)` unless a public API already existed.
4. Do not introduce new production `unwrap` / `expect` paths during extraction.
5. Keep module names discoverable from the feature name an agent would search for.

## TDD / Refactor Steps

### Step 1: Add a runner module map

Record existing major sections in `synthesized.rs` before extraction.

### Step 2: Extract one synthesized family at a time

After each extraction, run focused tests with non-zero counts, for example:

```bash
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-cli test_runner::synthesized -- --nocapture
CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-cli test_runner::executor -- --nocapture
```

### Step 3: Run CLI integration tests for generated rows

Run any affected `ash-cli` test binaries for test command JSON/human output.

### Step 4: Codex review

Ask Codex to verify that filtering, fail-fast, synthesized result identities, repro artifacts, and JSON output are unchanged.


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
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-cli
  - CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets --all-features -- -D warnings
  - python3 tools/dev/rust_file_size_report.py --markdown > /tmp/phase137-task1383-size.md
checklist:
  - [ ] Refactor is behavior-preserving
  - [ ] Public API paths preserved or deliberately documented
  - [ ] ash-cli tests pass
  - [ ] ash-cli clippy is clean
  - [ ] Formatting and diff checks pass
  - [ ] Size report shows intended reduction or documented exception
  - [ ] Codex final review reports no blocking issues
```


## Dependencies for Next Task

This task outputs:
- Synthesized test-runner logic split by generated-case family.
- Smaller files for contract/policy/obligation/small-world/law work.

Required by:
- TASK-1387: Module-size closeout and final audit.

## Notes

- This task should be committed independently.
- If any split exposes a genuine semantic bug, stop and create a follow-on bug task rather than hiding behavior changes in the refactor.
- End with Codex review for code quality, semantic preservation, style, and size-budget compliance.
