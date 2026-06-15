# SPEC-085: Proof-Producing Synthesis Todo Spec

**Status:** Deferred / To-Spec
**Date:** 2026-06-15
**Builds on:** [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Plan:** [PLAN-149: Proof-Producing Synthesis Todo Spec](../plan/PLAN-149-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)

## Summary

Record proof-producing synthesis as a separate non-test evidence family and defer implementation until symbolic/proof artifact trust boundaries are designed.

## Motivation

Phase 145 made empirical law evidence explicit but intentionally left several important `ash test` gaps visible. This specification defines the next scoped slice for those gaps while preserving the project rule that Ash law/test/proof authors and executors validate supported behavior through an Ash executable, not through Cargo or Rust test harnesses.

## Evidence Boundary

This packet is documentation-only. It records future proof-producing synthesis as a non-test evidence family and must not add parser syntax, solver calls, proof checking, or runner behavior.

A later implementation-grade proof-synthesis spec must distinguish:

1. **Implementation health** — Rust commands such as `cargo test`, `cargo clippy`, and `cargo fmt`; useful and required for implementers.
2. **Candidate Ash final surface** — direct invocations of an Ash-under-test executable for any user-facing command behavior.
3. **Release/install parity** — ordinary `ash` on PATH after install/release catches up.

`cargo run -p ash-cli -- test ...` is never final-surface evidence for future user-facing proof behavior.

## Scope

### In Scope

- Documentation-only classification of proof-producing synthesis as future non-test evidence.
- Trust-boundary questions for symbolic/solver/proof-term artifacts.
- To-spec criteria for a later implementation packet.

### Non-Goals

- implementation of symbolic execution
- solver integration
- proof checker
- proof-producing synthesis runtime
- changes to `ash test` beyond documentation

## Required Agent Skills

Implementation agents must load and follow:

- `rust-skills` for Rust code, public APIs, proptest coverage, error handling, and clippy-clean implementation.
- `ash-language-feature-spec-writing` for Ash surface/parser/typechecker/runner contracts and final-surface Ash examples.
- `test-driven-development` when implementing code slices: write failing Rust/Ash-facing tests before production changes.
- `verification-before-completion` before marking any task complete.
- `systematic-debugging` for any unexpected runner, parser, or property failure.

## Examples

Future syntax is intentionally illustrative only:

```ash
proof associativity(...) {
    by solver z3
}

proof safety(...) {
    by symbolic { produce proof_artifact }
}
```

This phase must not implement these forms; it records the future evidence-family boundary.

## Result and Reporting Requirements

- JSON output must remain machine-readable and stable enough for later orchestration.
- Unsupported cases must be `deferred`, `untested`, or explicit errors; they must not be counted as passing evidence.
- Repro artifacts must include enough data for a direct `$ASH_UNDER_TEST test ...` replay when the phase owns execution behavior.
- Human output should summarize the new capability without hiding caveats.

## Implementation Tasks

- [TASK-1482](../plan/tasks/TASK-1482-proof-producing-synthesis-landscape.md): Document proof-producing synthesis landscape
- [TASK-1483](../plan/tasks/TASK-1483-proof-evidence-family-boundary.md): Define future proof evidence family boundary
- [TASK-1484](../plan/tasks/TASK-1484-proof-producing-synthesis-deferred-closeout.md): Close deferred todo-spec packet

## Changelog

### 2026-06-15

- Created this planning specification and registered PLAN-149 / TASK-1482 through TASK-1484.
