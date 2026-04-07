# TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit

## Status: ✅ Complete

## Description

Run an explicit audit showing that `SPEC-025` remains compatible with both `SPEC-004` big-step semantics and the current implementation evidence recorded in `MCE-006`. The goal is not to prove full implementation closure, but to ensure `SPEC-025` is faithful, non-contradictory, and honest about where runtime support is partial or still missing.

This is docs/spec-audit work only.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)

## Dependencies

- ✅ [TASK-424: SPEC-025 Faithfulness and Compatibility Contract](TASK-424-spec-025-faithfulness-and-compatibility-contract.md)
- ✅ [TASK-425: SPEC-025 Rule-Schema and Helper-Boundary Consolidation](TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md)
- ✅ [TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)

## Requirements

### Functional Requirements

1. Produce one compatibility matrix from `SPEC-025` sections/claims to the relevant `SPEC-004` contracts.
2. Produce one compatibility matrix from `SPEC-025` runtime-facing claims to the frozen `MCE-006` evidence packet.
3. For each audited row, classify the current state as one of:
   - directly compatible / directly supported,
   - compatible but reconstructed or approximated,
   - compatible but weak/missing implementation support,
   - wording change required to avoid overclaim.
4. Cover at minimum:
   - terminal outcome reconstruction,
   - helper-boundary ownership,
   - `Receive` blocking/fallthrough behavior,
   - `Par` interleaving and terminal aggregation,
   - spawned-child completion/control ownership,
   - cumulative carrier claims for `Ω`, `π`, `T`, and `ε̂`.
5. State the final conservative verdict on whether `SPEC-025` is compatible with both big-step semantics and current implementation evidence.

### Non-Functional Requirements

1. Keep the audit conservative and evidence-based.
2. Do not confuse semantic compatibility with full implementation closure.
3. Use repo-relative links throughout.
4. Call out weak/missing support explicitly rather than smoothing it over.

## Audit Result

This task freezes two conservative matrices:

1. a `SPEC-025` → `SPEC-004` compatibility matrix for the normative semantic contract, and
2. a `SPEC-025` runtime-facing claims → `MCE-006` evidence matrix for current implementation support.

The audit conclusion is conservative:

- `SPEC-025` is compatible with `SPEC-004` as a small-step refinement/presentation of the same workflow semantics.
- `SPEC-025` is also compatible with the frozen `MCE-006` runtime evidence packet, but only if runtime-facing wording stays conservative.
- Current runtime evidence supports some `SPEC-025` claims directly, supports others only by reconstruction or approximation, and leaves several cumulative-carrier / retained-completion / explicit-`Par`-machine claims weak or missing.
- Therefore the correct verdict is compatibility without full implementation closure.

## SPEC-025 → SPEC-004 Compatibility Matrix

Classification vocabulary used below:

- directly compatible / directly supported
- compatible but reconstructed or approximated
- compatible but weak/missing implementation support
- wording change required to avoid overclaim

| `SPEC-025` section or claim | `SPEC-004` contract/evidence | Classification | Audit notes |
|---|---|---|---|
| [§1.3.2 terminal outcome reconstruction](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#132-preserved-spec-004-compatibility-constraints) and [§4 terminal projection](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#4-terminal-projection-and-big-step-correspondence) | `WorkflowOutcome ::= Return(...) | Reject(...)` in [SPEC-004 §2](../../spec/SPEC-004-SEMANTICS.md#2-semantic-domains) and the workflow judgment in [§3.1](../../spec/SPEC-004-SEMANTICS.md#31-workflow-big-step-judgment) | directly compatible / directly supported | The small-step terminal forms `Returned(...)` / `Rejected(...)` reconstruct the same big-step terminal outcome classes and keep `Ω`, `π`, `T`, and terminal effect projection aligned with the existing `Return(...)` / `Reject(...)` contract. |
| [§1.3.2 helper-boundary ownership](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#132-preserved-spec-004-compatibility-constraints) and [§3.4 helper relations](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#34-helper-relations-and-ownership-boundaries) | helper-boundary contract in [SPEC-004 §3.4](../../spec/SPEC-004-SEMANTICS.md#34-helper-relations), especially `select_receive_outcome(...)` and `combine_parallel_outcomes(...)` | directly compatible / directly supported | `SPEC-025` preserves the ownership boundary instead of flattening helper semantics into presentation-order machine rules. That is consistent with `SPEC-004`'s explicit helper-relations contract. |
| [§1.3.2 receive blocking/fallthrough semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#132-preserved-spec-004-compatibility-constraints), [§5.2 blocked/suspended](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#52-blocked-or-suspended), and [§7.5 receive rules](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#75-receive-and-concurrency-rules) | `RECEIVE-SELECTED` / `RECEIVE-FALLBACK` / `RECEIVE-FALLTHROUGH` / `RECEIVE-REJECT` in [SPEC-004 §4.1](../../spec/SPEC-004-SEMANTICS.md) and helper laws in [§6.2](../../spec/SPEC-004-SEMANTICS.md#62-receive-selection) | directly compatible / directly supported | `SPEC-025` keeps blocking receive distinct from stuckness and preserves fallback/fallthrough ownership at the receive helper boundary. That matches the big-step receive laws. |
| [§1.3.2 `Par` aggregation and determinism boundaries](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#132-preserved-spec-004-compatibility-constraints), [§7.5](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#75-receive-and-concurrency-rules), and [§8](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#8-concurrency-and-determinism-boundary) | `(PAR)` rule in [SPEC-004 §4](../../spec/SPEC-004-SEMANTICS.md) and `combine_parallel_outcomes(...)` laws in [§6.5](../../spec/SPEC-004-SEMANTICS.md#65-parallel-outcome-combination) | directly compatible / directly supported | `SPEC-025` preserves branch-local interleaving plus helper-backed aggregation and explicitly avoids recasting `Par` as left-to-right sequencing. That is the same determinism/aggregation boundary owned by `SPEC-004`. |
| [§1.3.2 spawned-child completion/control ownership boundaries](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#132-preserved-spec-004-compatibility-constraints), [§3.4](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#34-helper-relations-and-ownership-boundaries), and [§6 atomic boundaries](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#6-v1-atomic-boundaries) | control authority and completion payload contract in [SPEC-004 §3.5](../../spec/SPEC-004-SEMANTICS.md#35-control-authority-and-terminal-completion-payloads) and [§3.6](../../spec/SPEC-004-SEMANTICS.md#36-runtime-internal-supervisor-observation) | directly compatible / directly supported | `SPEC-025` keeps control authority, completion sealing, and retained completion observation at the existing runtime/supervisor boundary. It does not introduce new workflow syntax or move ownership away from `SPEC-004`. |
| [§2.2–§2.5 cumulative semantic carriers `Ω`, `π`, `T`, `ε̂`](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#22-metavariables-and-semantic-carriers) and [§4 correspondence table](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#4-terminal-projection-and-big-step-correspondence) | semantic domains and workflow outcome fields in [SPEC-004 §2](../../spec/SPEC-004-SEMANTICS.md#2-semantic-domains) and helper/join laws in [§6.4–§6.6](../../spec/SPEC-004-SEMANTICS.md) | directly compatible / directly supported | Semantically, `SPEC-025` is allowed to carry the same cumulative dimensions in configurations and labels. This is compatible with `SPEC-004` even though current runtime realization is incomplete. |
| [§9 informative runtime alignment notes](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#9-informative-runtime-alignment-notes) | informative runtime-neutral stance of `SPEC-004` plus helper/runtime ownership boundaries | wording change required to avoid overclaim | `SPEC-004` does not by itself justify strong implementation claims. The `SPEC-025` runtime-facing sections must therefore keep saying “compatible,” “partial,” “reconstructed,” or “weak/missing” where `MCE-006` does not show full direct runtime realization. |

## SPEC-025 Runtime-Facing Claims → MCE-006 Evidence Matrix

| `SPEC-025` runtime-facing claim | `MCE-006` evidence | Classification | Audit notes |
|---|---|---|---|
| Non-blocking `Receive` fallthrough, wildcard/timeout continuation, and blocking wait remain observable runtime behaviors as described in [SPEC-025 §9.1](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#91-receive-realization-evidence) | blocking/non-blocking/timeout receive evidence in [MCE-006 blocking receive analysis](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#21-blocking-receive-and-receive-path-control-traffic-waits), especially the notes on `wait_for_core_message(...)`, timeout fallback, and absence of semantic stuckness | directly compatible / directly supported | This is one of the strongest current runtime-alignment rows. `MCE-006` explicitly records blocking wait, timeout handling, and fallthrough-compatible behavior. |
| A coarse runtime distinction among active, blocked/suspended, terminal, and invalid/runtime-failure states is compatible with [SPEC-025 §9.2](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#92-coarse-runtime-outcome-state-evidence) | state-classification table and commentary in [MCE-006 TASK-402 section](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#3-distinguishing-active-blockedsuspended-terminal-and-invalidruntime-failure-state) | compatible but reconstructed or approximated | `MCE-006` supports the distinction only coarsely. The runtime does not package all classes in one uniform carrier, so the correspondence is reconstructed from mixed result/state surfaces. |
| Terminal outcome reconstruction is runtime-visible in the sense that success/failure classes can be related to `Returned(...)` / `Rejected(...)` | semantic-carrier mapping rows for `Returned(...)` and `Rejected(...)` plus observable-preservation table in [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#canonical-semantic-carrier--runtime-mapping-table) and [Phase 63 closeout](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#phase-63-closeout-observable-preservation-divergence-taxonomy-and-mce-007-handoff) | compatible but reconstructed or approximated | `Ok(Value)` directly supports the success class, but `Err(ExecError)` multiplexes rejection, runtime failure, and one explicit suspension boundary. So terminal reconstruction is possible, but not as one exact terminal payload carrier. |
| Helper-boundary ownership for receive selection, parallel aggregation, and control/completion remains a conservative runtime story rather than a flattened machine contract | repeated MCE-006 insistence that receive selection, `Par` aggregation, blocked-state realization, and completion/control remain helper/runtime correspondence work rather than proof of one explicit machine | compatible but reconstructed or approximated | `MCE-006` supports the ownership stance, but mostly as a descriptive runtime mapping story rather than one direct helper API packet visible end-to-end in the current interpreter. |
| `Par` is not implemented as obvious left-to-right sequential execution and retains a distinct aggregation boundary, as discussed in [SPEC-025 §9.4](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#94-current-parallel-realization-boundary) | `join_all(...)` / `Value::List(...)` evidence and the `Par` correspondence conclusions in [MCE-006 `Par` section](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#operational-correspondence-par-interleaving-branch-local-state-and-terminal-aggregation) | compatible but reconstructed or approximated | Current runtime evidence shows bulk async branch execution with one aggregation boundary. That is compatible with the semantic stance, but it is only an approximation of the fully explicit branch-step interleaving story. |
| `Par` helper-backed aggregation over cumulative carriers `Ω`, `π`, `T`, and `ε̂` should not be treated as fully evidenced by the current runtime | `Par` aggregation limitations in [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#4-current-terminal-aggregation-and-its-correspondence-limits), especially the explicit missing support for branch-local cumulative-state joins | compatible but weak/missing implementation support | `MCE-006` is explicit that successful child-value collation into `Value::List` does not amount to full helper-backed aggregation for the cumulative carriers. |
| Spawned-child control-authority lifecycle is directly evidenced, but retained completion packaging is only partial, as described in [SPEC-025 §9.3](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md#93-control-and-retained-completion-evidence) | control lifecycle and weak completion-payload evidence in [MCE-006 §4.1–§4.2](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#41-control-authority-lifecycle) | compatible but weak/missing implementation support | `ControlLinkRegistry` gives strong lifecycle evidence, but `MCE-006` remains explicit that full `CompletionPayload`-style retention and completion-observation waits are weak or missing on the inspected main path. |
| Semantic carrier `Ω` has some runtime realization | `Ω` mapping row and observable-preservation row in [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#canonical-semantic-carrier--runtime-mapping-table) | compatible but reconstructed or approximated | `Ω` is genuinely carried at runtime, but distributed across generic and role-scoped holders and not packaged as one authoritative terminal carrier. |
| Semantic carriers `π`, `T`, and `ε̂` exist as current runtime correspondents | mapping rows and closeout table in [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#canonical-semantic-carrier--runtime-mapping-table) and [Phase 63 closeout](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#phase-63-closeout-observable-preservation-divergence-taxonomy-and-mce-007-handoff) | wording change required to avoid overclaim | `MCE-006` marks `π`, `T`, and `ε̂` as weak/missing as authoritative cumulative carriers on the main execution path. `SPEC-025` may keep them normatively as semantic carriers, but its runtime-facing wording must not imply that the current interpreter already carries them directly and uniformly. |
| Blocked/suspended completion-observation waits or retained completion observation are current strong runtime evidence | completion-observation analysis in [MCE-006 TASK-402 section](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md#23-completion-observation-waits) | wording change required to avoid overclaim | `MCE-006` explicitly says completion-observation waits are weak/missing on the inspected main path. Any stronger wording would overclaim current evidence. |

## Conservative Verdict

Final verdict for TASK-426:

1. `SPEC-025` is semantically compatible with `SPEC-004`.
2. `SPEC-025` is also compatible with the frozen `MCE-006` runtime evidence packet, but only as a conservative semantics-plus-correspondence document.
3. The compatibility is strongest for terminal class reconstruction at the semantic/spec level, helper-boundary ownership, receive blocking/fallthrough semantics, and the fact that current `Par` execution is not merely obvious left-to-right sequencing.
4. The compatibility is weaker for runtime-side terminal packaging, blocked/suspended uniformity, `Par` as an explicit interleaving machine, spawned-child retained completion observation, and cumulative carriers beyond distributed/partial `Ω` support.
5. The main weak/missing runtime-support rows remain: authoritative runtime `π`, `T`, and `ε̂`; full helper-backed `Par` aggregation for `Ω` / `π` / `T` / `ε̂`; and full retained `CompletionPayload`-style completion packaging / completion-wait realization.
6. Therefore `SPEC-025` should be treated as faithful and compatible, but not as evidence that the current runtime already realizes the entire small-step carrier story directly.

## TDD Evidence

### Red

Before this task:
- compatibility is plausible, but not frozen in one explicit audit artifact.
- overclaim risk remains unless each runtime-facing statement is checked against MCE-006.

### Green

This task is complete when:
- a reader can trace every major SPEC-025 claim to SPEC-004 and MCE-006,
- any overclaim has been downgraded or corrected,
- the final compatibility verdict is explicit.

## Files

- Modify: `docs/plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

## Completion Checklist

- [x] TASK-426 task file updated with the explicit compatibility audit
- [x] SPEC-025 → SPEC-004 compatibility matrix created
- [x] SPEC-025 → MCE-006 compatibility matrix created
- [x] weak/missing implementation-support rows called out explicitly
- [x] final conservative verdict recorded

## Notes

Important constraints:
- Do not use this task to redesign runtime behavior.
- Do not imply full correspondence where MCE-006 records only partial realization.
