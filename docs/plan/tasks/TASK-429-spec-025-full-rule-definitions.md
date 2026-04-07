# TASK-429: SPEC-025 Full Rule Definitions

## Status: 📝 Planned

## Description

Expand `SPEC-025` from a faithful small-step rule inventory and contract document into a proof-usable canonical rule-definition spec for workflow small-step semantics. This task should not redesign accepted semantics; instead, it should formalize the already accepted workflow-form families into explicit rule definitions with clear premises, side conditions, and terminal/propagation structure so later proof and conformance work can cite one precise stepwise contract.

This remains docs/spec work only.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- [TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit](TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)

## Dependencies

- ✅ [TASK-427: SPEC-025 Faithful Closeout and Corpus Alignment](TASK-427-spec-025-faithful-closeout-and-corpus-alignment.md)
- 📝 [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)

## Requirements

### Functional Requirements

1. Revise `SPEC-025` so each canonical workflow-family accepted by the Phase 61 / Phase 66 corpus is represented by explicit rule definitions rather than only inventory/family wording.
2. Cover at minimum:
   - terminal and structural rules,
   - sequencing and propagation rules,
   - binding and branching rules,
   - capability / policy / obligation workflow rules,
   - modal/fallback rules,
   - receive and concurrency-facing rules,
   - terminal projection boundary back to `SPEC-004`.
3. Preserve the accepted v1 boundaries exactly:
   - workflow-first semantics remain primary,
   - expressions and patterns remain atomic where already frozen,
   - helper-owned boundaries remain helper-owned,
   - `Par` remains interleaving-compatible with helper-backed terminal aggregation.
4. Make premises, side conditions, and result forms explicit enough that later proof work can cite exact rule shapes rather than paraphrased prose.
5. Keep the document honest about what remains defined via helper contracts rather than local rule bodies.
6. Keep `SPEC-025` compatible with `SPEC-004` and the accepted `MCE-005` / `MCE-006` / `MCE-007` corpus.
7. Update `CHANGELOG.md` and any required planning/reference surfaces.

### Non-Functional Requirements

1. Do not reopen Phase 61 design decisions.
2. Do not introduce expression-level micro-step semantics in v1.
3. Keep repo-relative links throughout.
4. Distinguish normative rule definitions from informative commentary clearly.
5. Mark complete only if `SPEC-025` reads as a proof-usable rule-definition spec instead of only a faithful rule-family summary.

## TDD Evidence

### Red

Before this task:
- `SPEC-025` is a strong faithful small-step spec, but it still emphasizes rule inventory/family presentation over full canonical rule-definition form;
- downstream proof/conformance work would still need to reconstruct some exact rule shapes from cross-section prose;
- helper boundaries and accepted atomicity are frozen, but the workflow rule bodies are not yet uniformly packaged as one explicit inference-style contract.

### Green

This task is complete when:
- `SPEC-025` presents explicit canonical workflow small-step rule definitions for the accepted workflow families;
- propagation and terminal structure are explicit enough for proof and conformance citation;
- helper-owned boundaries remain explicit rather than flattened into machine detail;
- the document still stays faithful to `SPEC-004` and accepted Phase 61 / Phase 66 decisions.

## Files

- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/reference/formalization-boundary.md` (if rule-definition/proof-target wording needs refresh)
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] `SPEC-025` revised with explicit rule definitions
- [ ] canonical workflow families covered
- [ ] premises / side conditions / propagation shapes made explicit
- [ ] accepted v1 atomicity preserved
- [ ] helper-owned boundaries preserved explicitly
- [ ] compatibility with `SPEC-004` retained
- [ ] planning/reference surfaces updated as needed
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- a proof-usable workflow small-step rule-definition surface in `SPEC-025`.

Required by:
- TASK-430: Small-Step Helper Contracts and State Taxonomy
- TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh
- TASK-438: Canonical IR Semantics Corpus and Result Format

## Notes

Important constraints:
- Do not silently strengthen runtime claims while strengthening rule presentation.
- Do not collapse helper-owned nondeterminism into fake deterministic machine rules.
- Prefer explicit rule-shape clarity over informal paraphrase.
