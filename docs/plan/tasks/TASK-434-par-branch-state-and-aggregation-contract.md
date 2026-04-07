# TASK-434: `Par` Branch-State and Aggregation Contract

## Status: ✅ Complete

## Description

Freeze the exact semantic/runtime contract for `Par` branch-local state and helper-backed aggregation. This task should turn the currently accepted but still partially packaged `Par` story into one explicit contract that later runtime work can implement against: what semantic carriers are local to a branch, how successful and unsuccessful branch outcomes combine, which parts of aggregation remain helper-owned, and what counts as conformance when interleaving order is not fixed.

This remains contract/spec work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [SPEC-026: Implementation Conformance Contract](../../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-403: `Par` Interleaving, Branch State, and Aggregation Correspondence](TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md)
- [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)
- [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Dependencies

- ✅ [TASK-403: `Par` Interleaving, Branch State, and Aggregation Correspondence](TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md)
- ✅ [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)
- ✅ [TASK-432: Semantic Execution Record and Terminal Projection Contract](TASK-432-semantic-execution-record-and-terminal-projection-contract.md)

## Requirements

### Functional Requirements

1. Define one explicit `Par` branch-state and aggregation contract that later runtime work can implement directly.
2. State clearly which semantic carriers are branch-local during `Par` evaluation, including at minimum the treatment of `Ω`, `π`, `T`, `ε̂`, and branch terminal payloads.
3. Define how branch-local carriers combine under:
   - all-success completion,
   - branch failure/rejection,
   - blocked/suspended branches,
   - mixed and helper-owned concurrent outcomes.
4. Preserve the accepted semantics:
   - interleaving-compatible progress,
   - helper-backed aggregation,
   - no fake left-to-right sequential collapse,
   - no accidental scheduler theorem.
5. State the conformance rule for implementations when exact branch execution order differs but the allowed semantic contract is preserved.
6. Keep the contract compatible with `SPEC-004`, `SPEC-025`, and the execution-record contract from TASK-432.
7. Update planning/reporting/reference surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not implement runtime `Par` changes here.
2. Do not overclaim determinism beyond the already accepted helper-boundary model.
3. Use repo-relative links throughout.
4. Distinguish normative aggregation law from informative current-runtime evidence.
5. Mark complete only if TASK-435 can implement against this contract without rediscovering `Par` semantics from MCE prose.

## TDD Evidence

### Red

Before this task:
- `Par` is semantically accepted as interleaving plus helper-backed aggregation, but the exact branch-local carrier and aggregation contract remains only partially packaged for implementation follow-on;
- current runtime evidence shows useful concurrency, but not yet a fully frozen contract for cumulative-state aggregation;
- alternate implementations would still need to infer too much from MCE alignment prose.

### Green

This task is complete when:
- one explicit `Par` branch-state/aggregation contract exists;
- branch-local and combined carrier rules are stated clearly;
- implementation conformance for nondeterministic/interleaved branch execution is explicit;
- the contract is ready for direct runtime implementation follow-on.

## Files

- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md` (if helper-aggregation wording needs alignment)
- Modify: `docs/reference/semantic-execution-record-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] explicit `Par` branch-state/aggregation contract added
- [x] branch-local carrier ownership defined
- [x] aggregation behavior for success/failure/blocked cases defined
- [x] helper-owned nondeterminism boundary preserved
- [x] compatibility with TASK-432 execution-record contract preserved
- [x] planning/reference surfaces updated
- [x] `CHANGELOG.md` updated

## Completion Notes

TASK-434 is completed as a docs/spec/reference/planning contract pass.

The normative `Par` contract now lives directly in
[SPEC-025](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md): branch-local `Γ`, `Ω`, `π`, `T`,
`ε̂`, and terminal payload ownership are frozen explicitly for live `ParState(bs)` evaluation; helper-
backed aggregation is defined for all-success, mixed success/rejection, and blocked/nonterminal branch
collections; and the conformance rule now states what may vary across implementations (interleaving,
helper-owned concurrent rejection/trace merge latitude, runtime packaging) versus what must remain
semantically exact.

[docs/reference/semantic-execution-record-contract.md](../../reference/semantic-execution-record-contract.md)
now records the matching TASK-432 compatibility note for branch-local execution records and enclosing
aggregate execution-record projection, so TASK-435 can implement runtime `Par` aggregation directly
against this contract instead of reconstructing it from MCE prose.

This task remains contract-first only. No runtime `Par` implementation was changed here.

## Dependencies for Next Task

This task outputs:
- the frozen `Par` branch-state and aggregation contract for runtime implementation follow-on.

Required by:
- TASK-435: `Par` Runtime Aggregation Realization
- TASK-438: Canonical IR Semantics Corpus and Result Format
- TASK-439: Differential Conformance Harness (Rust First)

## Notes

Important constraints:
- Keep `Par` helper-backed, not scheduler-specific.
- Do not let runtime convenience collapse semantic branch-local carriers into undocumented shortcuts.
- Prefer exact contract wording over suggestive prose.
