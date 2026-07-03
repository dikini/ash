---
id: docs.notes.index
title: Ash Design Note Orientation Index
kind: orientation-index
status: active
authority: navigational
last_verified: 2026-07-03
---

# Ash design note orientation index

This index helps humans and agents choose the right design notes before editing specs, plans, or code. It combines a small topic ontology with flexible tags. Topics are for placement; tags are for retrieval.

## How to use this index

1. Start with the read path that matches the task.
2. Use `Primary topic` to find the conceptual cluster.
3. Use `Tags` for cross-cutting concerns such as `grammar`, `semantics`, `references`, `diagnostics`, or `authority`.
4. Check `Role` before treating a note as current guidance. Historical and superseded notes are context, not the implementation target.

## Topic ontology

- `ambient-computation`: ambient monad, handlers, effects, rows, and semantic anchors.
- `contracts`: contract semantics, lowering, diagnostics, trace contracts, and implementation handoffs.
- `runtime`: runtime organization, host/FFI, bottom, observability, and operational behavior.
- `workflow`: workflow forms, obligations, participants, and process/workflow interpretation.
- `type-system`: type-level constructs, nominals, newtypes, protocols, and type computation.
- `tooling`: MCP/LSP/benchmark/evaluation notes.
- `memory`: memory regions, ownership, and utilization.
- `general`: notes that are local, historical, or cross-cutting without a stronger placement.

## Tag vocabulary

Common tags include: `grammar`, `syntax`, `semantics`, `type-system`, `effect-system`, `core-ir`, `runtime`, `diagnostics`, `authority`, `evidence`, `references`, `snapshots`, `trace`, `temporal`, `testing`, `tooling`, `workflow`, `implemented`, `current-state`, `target-state`, `deferred`, and `orientation`.

## Read paths

### Implement Core contract predicate artifacts

1. [NOTE-031](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
2. [NOTE-033](NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
3. [NOTE-034](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
4. [PLAN-165](../plan/PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)

### Implement temporal/trace contract monitors

1. [NOTE-035](NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
2. [NOTE-034](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
3. [PLAN-165](../plan/PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)

### Work on ambient computation, handlers, and effect identity

1. [NOTE-013](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
2. [NOTE-020](NOTE-020-COMPUTATION-ROW-TAXONOMY.md)
3. [NOTE-022](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
4. [NOTE-023](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
5. [NOTE-025](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
6. [PLAN-180](../plan/PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md) for the target-doc cleanup that fences older capability-binding and WorkflowForm material.
7. [PLAN-182](../plan/PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md) for the implemented Core computation model slice: target `fn`, direct-style `do { ... }`, and callable row metadata.
8. [PLAN-183](../plan/PLAN-183-OPERATION-AUTHORITY-MODEL.md) for the current operation authority model: operation identities are impl/type-qualified, rows require authority without granting it, and non-operation row families keep distinct discharge rules.
9. [PLAN-184](../plan/PLAN-184-HANDLER-PROVIDER-SEMANTICS.md) for executable handler/provider frame semantics: raise/handle behavior, frame-stack lookup, missing discharge, and shadowing.

### Work on target-Ash convergence or stale-doc cleanup

1. [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
2. [NOTE-018](NOTE-018-BOUNDARY-DISCIPLINE.md)
3. [NOTE-019](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md)
4. [NOTE-022](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
5. [NOTE-023](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
6. [NOTE-025](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
7. [PLAN-180](../plan/PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md)
8. [PLAN-181](../plan/PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md) for current-state-vs-historical classification of legacy authority vocabulary.
9. [PLAN-182](../plan/PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md) for direct-style Core computation conformance.
10. [PLAN-183](../plan/PLAN-183-OPERATION-AUTHORITY-MODEL.md) for operation authority and row-family discharge consistency.
11. [PLAN-184](../plan/PLAN-184-HANDLER-PROVIDER-SEMANTICS.md) for handler/provider operational semantics and shadowing consistency.
12. [PLAN-185](../plan/PLAN-185-SURFACE-FUNCTION-LANGUAGE.md) for function-first target entry syntax and workflow compatibility/profile routing.
13. [PLAN-186](../plan/PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md) for CLI user-path conformance for function-first entry sources.
14. [PLAN-187](../plan/PLAN-187-SURFACE-RECORD-EXPRESSIONS.md) for structural record expressions in function-first Ash.
15. [PLAN-188](../plan/PLAN-188-SURFACE-MATCH-CONSTRUCTOR-SCRUTINEES.md) for ADT constructor expressions as ordinary match scrutinees in function-first Ash.
16. [PLAN-189](../plan/PLAN-189-SURFACE-MATCH-ORDINARY-SCRUTINEES.md) for call, field, and binary expressions as ordinary match scrutinees in function-first Ash.
17. [PLAN-190](../plan/PLAN-190-SURFACE-DO-EXPRESSION-STATEMENTS.md) for expression statements in unified direct-style `do`.
18. [PLAN-191](../plan/PLAN-191-SURFACE-BLOCK-EXPRESSIONS.md) for nested ordinary block expressions and block expression statements.
19. [PLAN-192](../plan/PLAN-192-SURFACE-POSTFIX-PROJECTION.md) for postfix field projection on ordinary primary expressions.
20. [PLAN-193](../plan/PLAN-193-SURFACE-TUPLE-ADT-EXPRESSIONS.md) for tuple-payload ADTs in function-first Ash.

### Change target handler/effect/operation syntax

1. [SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md)
2. [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
3. [NOTE-021](NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
4. [NOTE-022](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
5. [NOTE-023](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
6. [NOTE-025](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
7. [TASK-1692](../plan/tasks/TASK-1692-target-operation-row-syntax-alignment.md)

## Document table

| Document | Status | Primary topic | Tags | Role | Read with |
|---|---|---|---|---|---|
| [MCP-BENCHMARK-RESULTS.md](MCP-BENCHMARK-RESULTS.md) | unspecified | ambient-computation | effect-system | design note | — |
| [MCP-HERMES-INTEGRATION.md](MCP-HERMES-INTEGRATION.md) | unspecified | tooling | tooling | design note | — |
| [MCP-SPIKE-RESULTS.md](MCP-SPIKE-RESULTS.md) | - `ash_mcp_health` tool reports status, version, and available tool names. | tooling | tooling | design note | — |
| [NOTE-001-WORKFLOW-COMPUTATION-TYPE.md](NOTE-001-WORKFLOW-COMPUTATION-TYPE.md) | Future Opportunity | workflow | type-system, workflow | design note | — |
| [NOTE-002-DESIGN-027-EXECUTION-REVIEW.md](NOTE-002-DESIGN-027-EXECUTION-REVIEW.md) | Review Complete | general | orientation | design note | — |
| [NOTE-003-EXPR-LET-CORE-IR-GAP.md](NOTE-003-EXPR-LET-CORE-IR-GAP.md) | Spec Amendments Written | general | core-ir | design note | — |
| [NOTE-004-FN-CAPABILITY-WORKFLOW-EFFECT-TAXONOMY.md](NOTE-004-FN-CAPABILITY-WORKFLOW-EFFECT-TAXONOMY.md) | Superseded by NOTE-005 | ambient-computation | authority, effect-system | superseded/historical note | — |
| [NOTE-004-STDLIB-BUILTIN-GAP.md](NOTE-004-STDLIB-BUILTIN-GAP.md) | unspecified | general | orientation | design note | — |
| [NOTE-005-ACT-MONAD-UNIFYING-PURE-AND-EFFECTFUL.md](NOTE-005-ACT-MONAD-UNIFYING-PURE-AND-EFFECTFUL.md) | Superseded by SPEC-047 (normative spec) and PLAN-097 (implementation plan) | ambient-computation | effect-system | superseded/historical note | — |
| [NOTE-006-C3C-ACTENV-EXPOSURE-DESIGN.md](NOTE-006-C3C-ACTENV-EXPOSURE-DESIGN.md) | Active design note for TASK-689C | runtime | core-ir, runtime | design note | — |
| [NOTE-006-WORKFLOW-AMBIENT-TYPING-AND-RUNTIME-FAILURE.md](NOTE-006-WORKFLOW-AMBIENT-TYPING-AND-RUNTIME-FAILURE.md) | Historical/current-state workflow ambient typing direction; target authority routes through rows/admission | ambient-computation | current-state, effect-system, type-system, workflow | design note / compatibility context | SPEC-096b; SPEC-098b; SPEC-100; PLAN-181 |
| [NOTE-007-RUNTIME-ENVIRONMENT-IDENTITY-AND-COMPONENTS.md](NOTE-007-RUNTIME-ENVIRONMENT-IDENTITY-AND-COMPONENTS.md) | Current-state runtime environment component model; capability/provider components are compatibility substrate | runtime | authority, core-ir, current-state, runtime | design note / compatibility context | SPEC-096b; SPEC-099b; SPEC-100; PLAN-181 |
| [NOTE-007-context-send-sync-clone-cost-and-optimization-opportunities.md](NOTE-007-context-send-sync-clone-cost-and-optimization-opportunities.md) | Active implementation note for TASK-689D follow-on runtime work | runtime | runtime, semantics | design note | — |
| [NOTE-008-OPERATIONAL-BOTTOM-AND-SCOPED-ERROR-HANDLING.md](NOTE-008-OPERATIONAL-BOTTOM-AND-SCOPED-ERROR-HANDLING.md) | Draft | runtime | runtime | design note | — |
| [NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md](NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md) | Superseded by NOTE-022 / NOTE-023 / NOTE-025 (interface/impl/handler model) | ambient-computation | authority, effect-system | superseded/historical note | NOTE-022; NOTE-023; NOTE-025 |
| [NOTE-010-WORKFLOW-FORM-PRECHECK-QUESTIONS.md](NOTE-010-WORKFLOW-FORM-PRECHECK-QUESTIONS.md) | Historical / superseded WorkflowForm-era Q&A backlog; no revival | ambient-computation | core-ir, evidence, runtime, trace, workflow | historical design note | SPEC-096b; SPEC-098b; SPEC-099; TASK-1804 |
| [NOTE-011-TYPE-LEVEL-PROTOCOLS-CAPABILITY-AUTHORITY-AND-DISTRIBUTED-PARTICIPANTS.md](NOTE-011-TYPE-LEVEL-PROTOCOLS-CAPABILITY-AUTHORITY-AND-DISTRIBUTED-PARTICIPANTS.md) | Exploratory protocol note with historical/current-state capability-binding vocabulary fenced for target planning | ambient-computation | authority, effect-system, type-system | design note / historical authority vocabulary | NOTE-022; NOTE-023; NOTE-025; PLAN-180 |
| [NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md](NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md) | unspecified | general | core-ir | design note | — |
| [NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md) | Living document — exploration in progress | ambient-computation | effect-system | living design note | NOTE-020; NOTE-022; SPEC-096b; SPEC-097b |
| [NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md) | Closed as design gap register — resolved by NOTE-027 through NOTE-035; implementation handoff tracked by PLAN-165 | contracts | contract | convergence note | NOTE-027..NOTE-035; PLAN-165 |
| [NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md) | Living document — exploration in progress | general | orientation | living design note | — |
| [NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md) | Living document — exploration in progress | runtime | runtime | living design note | — |
| [NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md) | Living document — exploration in progress | memory | ownership | living design note | — |
| [NOTE-018-BOUNDARY-DISCIPLINE.md](NOTE-018-BOUNDARY-DISCIPLINE.md) | Living document — inventory in progress | ambient-computation | effect-system | living design note | — |
| [NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md](NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md) | Draft note — convergence map, not an implementation plan | general | orientation | design note | — |
| [NOTE-020-COMPUTATION-ROW-TAXONOMY.md](NOTE-020-COMPUTATION-ROW-TAXONOMY.md) | Promoted / partially realized -- taxonomy reflected in target specs and Core/CPS carriers | ambient-computation | effect-system, core-ir, type-system, implemented, target-state | promoted taxonomy note | SPEC-096b; SPEC-097b; SPEC-098b; SPEC-099; SPEC-100 |
| [NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md](NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md) | Living document -- syntax exploration for follow-up specs | ambient-computation | effect-system, grammar, syntax | living design note | — |
| [NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md) | Living document — declaration-side decision captured; dispatch-side open | ambient-computation | effect-system | living design note | NOTE-023; NOTE-025; SPEC-096b |
| [NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md) | Living document — dispatch-side direction captured; open questions tracked | ambient-computation | core-ir, effect-system | living design note | — |
| [NOTE-024-HOST-FFI-AND-EXTERN.md](NOTE-024-HOST-FFI-AND-EXTERN.md) | Living document — consolidated host/FFI design space; `extern` reserved but | runtime | runtime | living design note | — |
| [NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md) | Living document — design direction captured; revises the identity model from | ambient-computation | core-ir, effect-system | living design note | NOTE-022; NOTE-023; NOTE-026; SPEC-096b |
| [NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md](NOTE-026-NEWTYPE-AND-PHANTOM-TYPES.md) | Living document — design direction captured; derivation deferred | general | core-ir, type-system | living design note | — |
| [NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md](NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md) | Living document — design direction captured; resolves NOTE-014 GAP 1 (blame) and | contracts | contract, core-ir, diagnostics | convergence note | — |
| [NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md](NOTE-028-PURITY-EVALUATION-MODES-AND-CONTRACT-TIMING.md) | Living document — design direction captured; resolves NOTE-014 GAP 4 and | contracts | contract, core-ir | convergence note | — |
| [NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md](NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md) | Living document — design direction captured; resolves NOTE-014 GAP 6 | contracts | contract, core-ir, diagnostics | convergence note | — |
| [NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md](NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md) | Living document — design direction captured; resolves NOTE-014 GAP 2 | general | core-ir | convergence note | — |
| [NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md](NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md) | Living document — design direction captured; resolves NOTE-014 §13 open question 1 and NOTE-030 §9 open question 3 | contracts | contract, core-ir, semantics, snapshots | convergence note | — |
| [NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md](NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md) | Living document — design direction captured; resolves NOTE-014 GAP 7 | contracts | contract, core-ir | convergence note | — |
| [NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md](NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md) | Living document — design direction captured; resolves NOTE-014 GAP 9 | contracts | contract, core-ir | convergence note | NOTE-031; NOTE-034; SPEC-098b; SPEC-100; PLAN-165 |
| [NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md](NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md) | Living document — design direction captured; resolves NOTE-014 GAP 8 | contracts | authority, contract, core-ir | convergence note | NOTE-033; NOTE-035; SPEC-096b; SPEC-099 |
| [NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md](NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md) | Living document — design direction captured; resolves NOTE-014 GAP 5 | contracts | contract, core-ir, temporal, trace | convergence note | NOTE-034; SPEC-098b; SPEC-099; PLAN-165 |
| [PHASE-142-PERFORMANCE-BENCHMARK.md](PHASE-142-PERFORMANCE-BENCHMARK.md) | unspecified | tooling | tooling | design note | — |
| [PHASE-143-MCP-CROSS-LANGUAGE-EVALUATION.md](PHASE-143-MCP-CROSS-LANGUAGE-EVALUATION.md) | unspecified | tooling | tooling | design note | — |
| [diagnostic-span-default-tech-debt.md](diagnostic-span-default-tech-debt.md) | unspecified | general | diagnostics | design note | — |
