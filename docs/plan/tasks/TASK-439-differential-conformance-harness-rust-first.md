# TASK-439: Differential Conformance Harness (Rust First)

## Status: 📝 Planned

## Description

Build the first differential conformance harness against the canonical semantics corpus, starting with the Rust implementation. This task should turn the Phase 67 contract work into a runnable verification surface: execute canonical IR corpus cases against the Rust runtime, serialize results into the canonical result format, and compare them against expected outcomes or allowed outcome sets where bounded nondeterminism applies.

This is real implementation/test-infrastructure work.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)

## Dependencies

- 📝 [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- 📝 [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)
- ✅ [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- ✅ [TASK-435: `Par` Runtime Aggregation Realization](TASK-435-par-runtime-aggregation-realization.md)
- ✅ [TASK-436: Completion-Payload Parity Contract](TASK-436-completion-payload-parity-contract.md)
- 📝 [TASK-437: Retained-Completion Parity Follow-On](TASK-437-retained-completion-parity-follow-on.md)

## Requirements

### Functional Requirements

1. Implement the first differential conformance harness using the Rust implementation as the initial execution target.
2. The harness must:
   - load canonical IR corpus cases,
   - execute them against the Rust runtime/interpreter,
   - serialize runtime results into the canonical result format,
   - compare actual results to expected results or allowed-outcome sets.
3. The harness must handle bounded nondeterminism honestly, especially for `Par`, receive/blocking behavior, and retained completion/control observations where the contract allows multiple valid outcomes.
4. Add tests or harness fixtures demonstrating comparison of at least:
   - exact deterministic cases,
   - allowed-set nondeterministic cases,
   - failure/rejection cases,
   - runtime-observable retained completion/control cases where applicable.
5. Keep the harness extensible so later Lean/reference integration can reuse the same corpus and format instead of inventing a second testing protocol.
6. Update docs/planning/reporting surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Start Rust-first; do not implement Lean execution here.
2. Prefer canonical file-backed corpus fixtures over one-off ad hoc tests.
3. Keep comparison output auditable and useful for debugging mismatches.
4. Mark complete only if the harness provides a real reusable conformance check rather than only one-off example tests.

## TDD Evidence

### Red

Before this task:
- the corpus and result format are planning targets only;
- there is no reusable differential conformance harness for Rust against the Phase 67 semantic contracts;
- future alternate implementations would have no shared comparison substrate.

### Green

This task is complete when:
- Rust can be run against canonical corpus cases through one reusable harness;
- actual results are normalized into the canonical format and checked against expected/allowed outcomes;
- the harness is ready for later Lean/reference extension.

## Files

- Create: `tests/differential/` fixtures and harness files as needed
- Create: `scripts/` support files as needed
- Modify: relevant Rust crate/test infrastructure files as needed
- Modify: `docs/reference/canonical-ir-semantics-corpus.md`
- Modify: `docs/reference/canonical-semantics-result-format.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing harness tests/fixtures

Add corpus fixtures and tests that require Rust results to be normalized and compared against the canonical format.

### Step 2: Implement Rust-first conformance harness

Add the reusable harness, normalization, and comparison logic.

### Step 3: Verify affected crate/test quality

Run at least:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- the repository now contains a reusable Rust-first differential conformance harness aligned with the Phase 67 contracts.

## Completion Checklist

- [ ] TASK-439 task file created
- [ ] canonical corpus fixtures wired to a harness
- [ ] Rust results normalized into canonical result format
- [ ] expected/allowed outcome comparison implemented
- [ ] tests added or updated
- [ ] docs/planning surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- the first reusable differential conformance harness for Ash.

Required by:
- TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Notes

Important constraints:
- Keep comparison semantics driven by TASK-428 and TASK-438, not by test convenience.
- Make nondeterminism explicit rather than hiding it with flaky tests.
- Prefer reusable normalization/comparison code over fixture-specific assertions.
