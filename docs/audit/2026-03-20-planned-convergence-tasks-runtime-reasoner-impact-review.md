# Planned Convergence Tasks Runtime-Reasoner Impact Review

## Scope

Reviewed the planned convergence tasks:

- [TASK-164](/home/dikini/Projects/ash/docs/plan/tasks/TASK-164-route-receive-through-main-parser.md)
- [TASK-165](/home/dikini/Projects/ash/docs/plan/tasks/TASK-165-align-check-decide-ast-contracts.md)
- [TASK-166](/home/dikini/Projects/ash/docs/plan/tasks/TASK-166-replace-placeholder-policy-lowering.md)
- [TASK-167](/home/dikini/Projects/ash/docs/plan/tasks/TASK-167-lower-receive-into-canonical-core-form.md)
- [TASK-168](/home/dikini/Projects/ash/docs/plan/tasks/TASK-168-align-type-checking-for-policies-and-receive.md)
- [TASK-169](/home/dikini/Projects/ash/docs/plan/tasks/TASK-169-unify-runtime-verification-context-and-obligation-enforcement.md)
- [TASK-170](/home/dikini/Projects/ash/docs/plan/tasks/TASK-170-implement-end-to-end-receive-execution.md)
- [TASK-171](/home/dikini/Projects/ash/docs/plan/tasks/TASK-171-align-runtime-policy-outcomes.md)
- [TASK-172](/home/dikini/Projects/ash/docs/plan/tasks/TASK-172-unify-repl-implementation.md)
- [TASK-173](/home/dikini/Projects/ash/docs/plan/tasks/TASK-173-implement-repl-type-reporting.md)

against the runtime-reasoner docs corpus:

- [Runtime-Reasoner Spec Handoff](/home/dikini/Projects/ash/docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md)
- [Runtime-to-Reasoner Interaction Contract](/home/dikini/Projects/ash/docs/reference/runtime-to-reasoner-interaction-contract.md)
- [Surface Guidance Boundary](/home/dikini/Projects/ash/docs/reference/surface-guidance-boundary.md)
- [Ash Language Terminology Guide](/home/dikini/Projects/ash/docs/design/LANGUAGE-TERMINOLOGY.md)
- [SPEC-004: Operational Semantics](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md)

Review protocol:

- [Runtime-Reasoner Separation Rules](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)

Review date: 2026-03-20

## Summary

The runtime-reasoner docs do not require scope changes for the existing parser, AST, lowering,
type-checking, verification, or runtime execution tasks. The new interaction-facing docs are
deliberately orthogonal to those planned convergence tasks.

The only tasks that need touch-point updates are the user-facing REPL tasks, which should carry the
new reference corpus when later implementation planning revisits surface guidance and observable
output wording. Those updates are reference-only, not scope changes.

No task is blocked by the runtime-reasoner docs corpus.

## Evidence Basis

The classification is driven by two explicit constraints in the new corpus:

- the runtime-reasoner handoff says the follow-up phase does not authorize parser, lowering,
  type-system, interpreter, capability-runtime, or CLI/REPL implementation changes
- the interaction contract and surface-guidance boundary keep projection, advisory outputs, and
  human-facing stage guidance separate from runtime-only observability and execution semantics

That means the reviewed convergence tasks still make sense in a reasoner-free runtime, so their
implementation scope stays intact even where later reference updates may be useful.

## Findings

### 1. TASK-164 is `unchanged`

Reference: [TASK-164](/home/dikini/Projects/ash/docs/plan/tasks/TASK-164-route-receive-through-main-parser.md)

Reasoning: This task is a parser dispatch fix for canonical `receive`. The runtime-reasoner corpus
adds no new parser surface or transport semantics. Its scope stays fully covered by the existing
surface-to-parser contract and SPEC-013.

### 2. TASK-165 is `unchanged`

Reference: [TASK-165](/home/dikini/Projects/ash/docs/plan/tasks/TASK-165-align-check-decide-ast-contracts.md)

Reasoning: The task aligns AST contracts for `check` and `decide`. The new runtime-reasoner docs do
not change those canonical forms, and they remain runtime/surface contracts rather than interaction
contracts.

### 3. TASK-166 is `unchanged`

Reference: [TASK-166](/home/dikini/Projects/ash/docs/plan/tasks/TASK-166-replace-placeholder-policy-lowering.md)

Reasoning: Policy lowering is still a runtime contract concern. The interaction contract and
surface-guidance boundary do not alter the lowering target or the policy shape.

### 4. TASK-167 is `unchanged`

Reference: [TASK-167](/home/dikini/Projects/ash/docs/plan/tasks/TASK-167-lower-receive-into-canonical-core-form.md)

Reasoning: `receive` now has explicit runtime-only framing and an interaction contract that excludes
projection and monitorability. The task scope stays the same, and the new runtime-reasoner corpus
does not require a scope or reference change for this lowering work.

### 5. TASK-168 is `unchanged`

Reference: [TASK-168](/home/dikini/Projects/ash/docs/plan/tasks/TASK-168-align-type-checking-for-policies-and-receive.md)

Reasoning: Type-checking for policies and `receive` remains a runtime/surface enforcement concern.
The new docs do not change the type rules or the task scope.

### 6. TASK-169 is `unchanged`

Reference: [TASK-169](/home/dikini/Projects/ash/docs/plan/tasks/TASK-169-unify-runtime-verification-context-and-obligation-enforcement.md)

Reasoning: Runtime verification context and obligation enforcement are explicitly runtime-only in
the new docs corpus. The task does not need scope or reference change.

### 7. TASK-170 is `unchanged`

Reference: [TASK-170](/home/dikini/Projects/ash/docs/plan/tasks/TASK-170-implement-end-to-end-receive-execution.md)

Reasoning: End-to-end `receive` execution remains a runtime behavior task. The interaction contract
explicitly keeps projection, advisory output, and monitor/exposes behavior outside runtime
execution semantics.

### 8. TASK-171 is `unchanged`

Reference: [TASK-171](/home/dikini/Projects/ash/docs/plan/tasks/TASK-171-align-runtime-policy-outcomes.md)

Reasoning: Runtime policy outcomes are still owned by runtime verification and interpreter
integration. The new docs clarify authority boundaries but do not change the outcome model.

### 9. TASK-172 is `reference-update-only`

Reference: [TASK-172](/home/dikini/Projects/ash/docs/plan/tasks/TASK-172-unify-repl-implementation.md)

Reasoning: REPL unification is unchanged in scope, but it now sits downstream from the runtime
observable behavior contract, the surface-guidance boundary, and the new interaction corpus. Any
later implementation pass should reference those docs to avoid conflating runtime observability with
reasoner projection.

### 10. TASK-173 is `reference-update-only`

Reference: [TASK-173](/home/dikini/Projects/ash/docs/plan/tasks/TASK-173-implement-repl-type-reporting.md)

Reasoning: REPL type reporting remains a runtime-observable behavior task. The new docs do not alter
the reporting behavior, but they do add context that should be referenced if the task later needs to
describe advisory versus authoritative workflow stages.

## Cross-Cutting Observation

The current convergence queue does not need new blocking dependencies from the runtime-reasoner
follow-up phase. The impact is mostly documentation hygiene and later reference alignment.

The only tasks with explicit follow-up pressure are the REPL tasks, because they are the most likely
to surface human-facing guidance later. Even there, the effect is reference-only, not scope-changing.

## Conclusion

No planned convergence task is blocked by the new runtime-reasoner docs corpus.

Most tasks are `unchanged`. A small subset is `reference-update-only` so later implementation work
can cite the new runtime-observable behavior and surface-guidance boundary without reopening the
docs-only phase.
