---
id: docs.spec.index
title: Ash Specification Orientation Index
kind: orientation-index
status: active
authority: navigational
last_verified: 2026-07-03
---

# Ash specification orientation index

This index helps humans and agents choose the right specifications before planning or implementing Ash features. It is navigational metadata, not a replacement for the specs themselves.

## How to use this index

1. Pick the current-state or target-state spec based on the task.
2. Use `Primary topic` for conceptual placement.
3. Use `Tags` for cross-cutting retrieval concerns such as `grammar`, `semantics`, `core-ir`, `runtime`, `testing`, or `references`.
4. Use `Read with` to avoid reading one spec in isolation when the rule spans grammar, type checking, IR, and runtime.

## Topic ontology

- `language-surface`: grammar, parser-facing syntax, and source-level forms.
- `type-system`: typing, type computation, interfaces, constraints, and summaries.
- `effect-system`: effects, rows, handlers, authority, and provider/handler admission boundaries.
- `core-ir`: Core Ash, Target IR, CPS IR, and operational/type-checking semantics.
- `runtime`: process/workflow runtime, observability, resources, and execution behavior.
- `testing`: laws, evidence, QuickCheck, mutation, coverage, and test orchestration.
- `tooling`: CLI, LSP, MCP, formatter, lint, Ashgrove, and reference maintenance tools.
- `contracts`: contract-specific semantics and Core/type/runtime integration.
- `general`: specs whose main role is local or historical.

## Tag vocabulary

Common tags include: `grammar`, `syntax`, `semantics`, `type-system`, `effect-system`,
`core-ir`, `runtime`, `diagnostics`, `authority`, `evidence`, `references`, `testing`,
`tooling`, `current-state`, `target-state`, `implemented`, `deferred`, and `orientation`.

## Status and role guide

- `current-state spec`: describes the current implementation boundary.
- `target-state spec`: describes the intended target design.
- `implemented spec`: has an MVP or accepted implementation slice.
- `deferred spec`: records a postponed or todo-spec scope.
- `normative spec`: can govern implementation if no more specific current/target distinction applies.

## Read paths

### Grammar, surface syntax, macros, or notation work

1. [SPEC-095a](SPEC-095a-CURRENT-GRAMMAR.md) for current parser behavior.
2. [SPEC-095b](SPEC-095b-TARGET-GRAMMAR.md) for target syntax.
3. [SPEC-095c](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md) for source-preserving AST, macros, notation, operator sections, and Phase 173/174 macro carriers/tooling boundaries.
4. [PLAN-174](../plan/PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md) for macro-aware tooling, summary identity, and callable-identity inference readiness.
5. [PLAN-173](../plan/PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md) for macro summary, token-tree, hygiene, and typed-macro task sequencing.
6. Relevant design notes tagged `grammar` in [NOTE-INDEX](../notes/NOTE-INDEX.md).

### Target handler/effect/operation syntax

1. [SPEC-095b](SPEC-095b-TARGET-GRAMMAR.md)
2. [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
3. [NOTE-021](../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
4. [NOTE-022](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
5. [NOTE-023](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
6. [NOTE-025](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
7. [TASK-1692](../plan/tasks/TASK-1692-target-operation-row-syntax-alignment.md)

### Target effect/type/IR/lowering planning

1. [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
2. [SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md)
3. [SPEC-098b](SPEC-098b-TARGET-IR.md)
4. [SPEC-098c](SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
5. [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
6. [SPEC-099](SPEC-099-CORE-LANGUAGE.md)
7. [SPEC-100](SPEC-100-CORE-TYPE-CHECKING.md)
8. [PLAN-178](../plan/PLAN-178-SOURCE-TO-CORE-ROW-LOWERING-BRIDGE.md) for the current explicit source callable row to Core metadata bridge and remaining non-authority caveats.
9. [PLAN-179](../plan/PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md) for the completed explicit row requirement admission/runtime wiring slice.
10. [PLAN-180](../plan/PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md) for the target-doc consistency cleanup that fences NOTE-009-era capability-binding and WorkflowForm material.
11. [PLAN-181](../plan/PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md) for the legacy authority vocabulary audit and current-state-vs-historical classification.
12. [PLAN-182](../plan/PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md) for the implemented Core computation model slice: target `fn`, direct-style `do { ... }`, and callable row metadata.

### Legacy authority vocabulary audit

1. [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
2. [SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md)
3. [SPEC-098b](SPEC-098b-TARGET-IR.md)
4. [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
5. [SPEC-100](SPEC-100-CORE-TYPE-CHECKING.md)
6. [NOTE-022](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
7. [NOTE-023](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
8. [NOTE-025](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
9. [PLAN-181](../plan/PLAN-181-LEGACY-AUTHORITY-VOCABULARY-AUDIT.md) for classification of older capability/provider specs.

### Current vs target Core/type/effect planning

1. [SPEC-096a](SPEC-096a-CURRENT-EFFECT-SYSTEM.md) and [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
2. [SPEC-097a](SPEC-097a-CURRENT-TYPE-SYSTEM.md) and [SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md)
3. [SPEC-098a](SPEC-098a-CURRENT-IR.md) and [SPEC-098b](SPEC-098b-TARGET-IR.md)
4. [SPEC-099a](SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md) and [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
5. [SPEC-099](SPEC-099-CORE-LANGUAGE.md) and [SPEC-100](SPEC-100-CORE-TYPE-CHECKING.md)
6. [NOTE-015](../notes/NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md) and [NOTE-019](../notes/NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md)

### Contract implementation work

1. [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
2. [SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md)
3. [SPEC-098b](SPEC-098b-TARGET-IR.md)
4. [SPEC-099](SPEC-099-CORE-LANGUAGE.md)
5. [SPEC-100](SPEC-100-CORE-TYPE-CHECKING.md)
6. [PLAN-165](../plan/PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)

## Document table

| Document | Status | Primary topic | Tags | Role | Read with |
|---|---|---|---|---|---|
| [SPEC-001-IR.md](SPEC-001-IR.md) | Draft | core-ir | core-ir, semantics | normative spec | — |
| [SPEC-002-SURFACE.md](SPEC-002-SURFACE.md) | Legacy/current-state surface baseline; capability invocation syntax is compatibility context | language-surface | current-state, grammar, references, surface | current-state compatibility spec | SPEC-095b; SPEC-096b; SPEC-098c; NOTE-022; NOTE-023; NOTE-025; PLAN-181 |
| [SPEC-003-TYPE-SYSTEM.md](SPEC-003-TYPE-SYSTEM.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-004-SEMANTICS.md](SPEC-004-SEMANTICS.md) | Historical/current-state Core semantics draft; target operational semantics superseded by SPEC-099b | core-ir | core-ir, semantics, current-state | historical/current-state spec | SPEC-098b; SPEC-099b; SPEC-100; PLAN-180 |
| [SPEC-005-CLI.md](SPEC-005-CLI.md) | unspecified | tooling | tooling | normative spec | — |
| [SPEC-006-POLICY-DEFINITIONS.md](SPEC-006-POLICY-DEFINITIONS.md) | Draft | language-surface | grammar, surface | normative spec | — |
| [SPEC-007-POLICY-COMBINATORS.md](SPEC-007-POLICY-COMBINATORS.md) | Draft | general | orientation | normative spec | — |
| [SPEC-008-DYNAMIC-POLICIES.md](SPEC-008-DYNAMIC-POLICIES.md) | Draft (Deferred) | general | deferred | deferred spec | — |
| [SPEC-009-MODULES.md](SPEC-009-MODULES.md) | Draft (Section 4.5 IO Module Tree - V1 Frozen) | general | orientation | normative spec | — |
| [SPEC-010-EMBEDDING.md](SPEC-010-EMBEDDING.md) | Draft (IO Provider References - V1 Frozen) | effect-system | authority, effect-system, references | normative spec | — |
| [SPEC-011-REPL.md](SPEC-011-REPL.md) | Draft | tooling | tooling | normative spec | — |
| [SPEC-012-IMPORTS.md](SPEC-012-IMPORTS.md) | Draft (IO Import Examples - V1 Frozen) | general | orientation | normative spec | — |
| [SPEC-013-STREAMS.md](SPEC-013-STREAMS.md) | Draft | runtime | runtime | normative spec | — |
| [SPEC-014-BEHAVIOURS.md](SPEC-014-BEHAVIOURS.md) | Draft | runtime | runtime | normative spec | — |
| [SPEC-015-TYPED-PROVIDERS.md](SPEC-015-TYPED-PROVIDERS.md) | Draft | runtime | authority, effect-system, runtime, type-system | normative spec | — |
| [SPEC-016-OUTPUT.md](SPEC-016-OUTPUT.md) | Draft | general | orientation | normative spec | — |
| [SPEC-017-CAPABILITY-INTEGRATION.md](SPEC-017-CAPABILITY-INTEGRATION.md) | Current-state compatibility; V1 IO capability boundary frozen, target authority superseded by row/provider admission docs | effect-system | authority, current-state, effect-system, references | current-state compatibility spec | SPEC-096b; NOTE-022; NOTE-023; NOTE-025; SPEC-100; PLAN-181 |
| [SPEC-018-CAPABILITY-MATRIX.md](SPEC-018-CAPABILITY-MATRIX.md) | Historical/current-state capability verification matrix; target admission superseded by computation-row discharge | runtime | authority, current-state, effect-system, runtime | historical/current-state compatibility spec | SPEC-096b; SPEC-100; PLAN-181 |
| [SPEC-019-ROLE-RUNTIME-SEMANTICS.md](SPEC-019-ROLE-RUNTIME-SEMANTICS.md) | Current-state role authority substrate; target role/operation authority is row-accounted admission evidence | runtime | authority, core-ir, current-state, effect-system, runtime, semantics | current-state compatibility spec | SPEC-096b; SPEC-100; PLAN-181 |
| [SPEC-020-ADT-TYPES.md](SPEC-020-ADT-TYPES.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) | Draft | runtime | runtime | normative spec | — |
| [SPEC-022-WORKFLOW-TYPING.md](SPEC-022-WORKFLOW-TYPING.md) | Current-state workflow typing/obligation substrate; provider-action capability syntax is compatibility context | runtime | current-state, runtime, type-system, workflow | current-state compatibility spec | SPEC-096b; SPEC-098b; SPEC-100; PLAN-181 |
| [SPEC-023-PROXY-WORKFLOWS.md](SPEC-023-PROXY-WORKFLOWS.md) | Draft | runtime | runtime | normative spec | — |
| [SPEC-024-CAPABILITY-ROLE-REDUCED.md](SPEC-024-CAPABILITY-ROLE-REDUCED.md) | Superseded historical capability-role surface; canonical only for legacy reduced surface context | runtime | authority, effect-system, grammar, references, runtime, surface | superseded/historical spec | SPEC-096b; NOTE-022; NOTE-025; PLAN-181 |
| [SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) | Draft | core-ir | core-ir, semantics | normative spec | — |
| [SPEC-026-IMPLEMENTATION-CONFORMANCE.md](SPEC-026-IMPLEMENTATION-CONFORMANCE.md) | Draft | contracts | contract | normative spec | — |
| [SPEC-027-PURE-FUNCTIONS.md](SPEC-027-PURE-FUNCTIONS.md) | Draft | general | orientation | normative spec | — |
| [SPEC-028-FUNCTION-CONSTRAINT-SYSTEM.md](SPEC-028-FUNCTION-CONSTRAINT-SYSTEM.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-029-LLM-STDLIB.md](SPEC-029-LLM-STDLIB.md) | Draft | general | orientation | normative spec | — |
| [SPEC-030-MODULE-TYPE-RESOLUTION.md](SPEC-030-MODULE-TYPE-RESOLUTION.md) | Draft (v3 -- revised after independent review) | type-system | type-system | normative spec | — |
| [SPEC-031-FIRST-CLASS-FUNCTIONS.md](SPEC-031-FIRST-CLASS-FUNCTIONS.md) | Draft | core-ir | core-ir, semantics | normative spec | — |
| [SPEC-032-MULTI-PARAMETER-INTERFACE-METHODS.md](SPEC-032-MULTI-PARAMETER-INTERFACE-METHODS.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-033-MULTI-PARAMETER-INTERFACES.md](SPEC-033-MULTI-PARAMETER-INTERFACES.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md](SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-035-ASSOCIATED-TYPES.md](SPEC-035-ASSOCIATED-TYPES.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-036-DERIVE-MECHANICS.md](SPEC-036-DERIVE-MECHANICS.md) | Draft | general | orientation | normative spec | — |
| [SPEC-037-TYPE-IMPL-QUOTATIONS.md](SPEC-037-TYPE-IMPL-QUOTATIONS.md) | Draft | type-system | type-system | normative spec | — |
| [SPEC-038-LANGUAGE-SERVER.md](SPEC-038-LANGUAGE-SERVER.md) | Draft (Partially Implemented — Local LSP MVP) | tooling | implemented, tooling, type-system | implemented spec | — |
| [SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md](SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md) | unspecified | tooling | tooling | normative spec | — |
| [SPEC-039-PARSER-TOOLING-INFRASTRUCTURE.md](SPEC-039-PARSER-TOOLING-INFRASTRUCTURE.md) | Draft | tooling | grammar, surface, tooling | normative spec | — |
| [SPEC-040-DIAGNOSTIC-INFRASTRUCTURE.md](SPEC-040-DIAGNOSTIC-INFRASTRUCTURE.md) | Draft | general | orientation | normative spec | — |
| [SPEC-041-ASH-LINT-LIBRARY.md](SPEC-041-ASH-LINT-LIBRARY.md) | Draft | tooling | tooling | normative spec | — |
| [SPEC-042-ASH-SOURCE-FORMATTER.md](SPEC-042-ASH-SOURCE-FORMATTER.md) | Draft | tooling | tooling | normative spec | — |
| [SPEC-043-INCREMENTAL-ANALYSIS.md](SPEC-043-INCREMENTAL-ANALYSIS.md) | Draft (Planned / Not Implemented) | tooling | core-ir, implemented, semantics, tooling | implemented spec | — |
| [SPEC-044-generic-builtin-fn.md](SPEC-044-generic-builtin-fn.md) | Draft | general | orientation | normative spec | — |
| [SPEC-045-ASH-WIKI.md](SPEC-045-ASH-WIKI.md) | Draft | general | orientation | normative spec | — |
| [SPEC-046-LEAN-REFERENCE.md](SPEC-046-LEAN-REFERENCE.md) | Legacy reference sketch | general | references | normative spec | — |
| [SPEC-047-ACT-MONAD.md](SPEC-047-ACT-MONAD.md) | Current-state Act substrate; capability-provider vocabulary is compatibility context for target rows/admission | core-ir | authority, core-ir, current-state, effect-system, semantics | current-state compatibility spec | SPEC-096b; SPEC-098b; SPEC-100; PLAN-181 |
| [SPEC-048-PROC-LIBRARY.md](SPEC-048-PROC-LIBRARY.md) | Draft | runtime | runtime | normative spec | — |
| [SPEC-049-PROCESS-RUNTIME-SEMANTICS.md](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md) | Current-state process runtime substrate; authority projection vocabulary maps to target row/admission evidence | runtime | authority, core-ir, current-state, runtime, semantics | current-state compatibility spec | SPEC-096b; SPEC-099b; SPEC-100; PLAN-181 |
| [SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md) | Draft | core-ir | core-ir, semantics | normative spec | — |
| [SPEC-051-WORKFLOW-SEMANTICS.md](SPEC-051-WORKFLOW-SEMANTICS.md) | Current-state workflow governance substrate; role/capability admission maps to target rows and ambient facts | runtime | authority, core-ir, current-state, runtime, semantics, workflow | current-state compatibility spec | SPEC-096b; SPEC-098b; SPEC-099b; SPEC-100; PLAN-181 |
| [SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) | Historical/current-state substrate; target authority vocabulary superseded by NOTE-022/023/025 | effect-system | authority, current-state, effect-system, references | historical/current-state spec | NOTE-022; NOTE-023; NOTE-025; PLAN-180 |
| [SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) | Historical/current-state resource substrate; target authority vocabulary superseded by NOTE-022/023/025 | runtime | authority, current-state, effect-system, references, runtime | historical/current-state spec | NOTE-022; NOTE-023; NOTE-025; PLAN-180 |
| [SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) | Implemented MVP (Phase 105) | type-system | implemented, type-system | implemented spec | — |
| [SPEC-055-MONAD-COMPREHENSION-SYNTAX.md](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) | Implemented MVP | language-surface | grammar, implemented, surface | implemented spec | — |
| [SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) | Implemented MVP / historical WorkflowForm language superseded for target planning | runtime | core-ir, effect-system, implemented, references, runtime, semantics, workflow | implemented spec / historical carrier framing | SPEC-096b; SPEC-098b; SPEC-099b; SPEC-100; TASK-1804; PLAN-180 |
| [SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) | Implemented MVP | core-ir | core-ir, implemented, semantics, type-system | implemented spec | — |
| [SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md) | Implemented MVP | core-ir | core-ir, implemented, semantics, type-system | implemented spec | — |
| [SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) | Implemented MVP | core-ir | core-ir, implemented, semantics, type-system | implemented spec | — |
| [SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md](SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md](SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md](SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md](SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) | Implemented MVP | type-system | implemented, type-system | implemented spec | — |
| [SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md](SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md) | Implemented MVP | general | implemented | implemented spec | — |
| [SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md) | Implemented alpha compatibility substrate; capability/provider vocabulary maps to target row/admission semantics | testing | authority, evidence, implemented, testing | implemented compatibility spec | SPEC-096b; SPEC-098b; SPEC-100; PLAN-181 |
| [SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md) | Implemented alpha runtime compatibility substrate; admission/grant vocabulary maps to target provider/resource evidence | testing | authority, evidence, grammar, implemented, runtime, surface, testing | implemented compatibility spec | SPEC-096b; SPEC-099b; SPEC-100; PLAN-181 |
| [SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md) | Implemented MVP | general | implemented, references | implemented spec | — |
| [SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md) | Implemented callable syntax; capability/provider references are compatibility notes for target row/admission semantics | type-system | grammar, implemented, surface, type-system | implemented spec / compatibility note | SPEC-096b; SPEC-097b; PLAN-181 |
| [SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md](SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) | Implemented MVP after TASK-986 closeout | tooling | implemented, tooling | implemented spec | — |
| [SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md](SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) | Accepted/Implemented | tooling | implemented, tooling | implemented spec | — |
| [SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md](SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md) | Implemented MVP | runtime | implemented, references, runtime | implemented spec | — |
| [SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md](SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md) | Implemented MVP | general | implemented | implemented spec | — |
| [SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md](SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md) | Implemented MVP | testing | evidence, implemented, testing | implemented spec | — |
| [SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md](SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md) | Implemented MVP | general | implemented | implemented spec | — |
| [SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md](SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md) | Implemented MVP | language-surface | grammar, implemented, surface | implemented spec | — |
| [SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md](SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md) | Implemented MVP | testing | evidence, implemented, testing, type-system | implemented spec | — |
| [SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md) | Implemented MVP (Phase 145) | testing | evidence, implemented, testing | implemented spec | [PLAN-145](../plan/PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md); TASK-1805 |
| [SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md](SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md) | Implemented MVP (Phase 146) | testing | evidence, implemented, testing | implemented spec | [PLAN-146](../plan/PLAN-146-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md); TASK-1805 |
| [SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md](SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md) | Implemented MVP | testing | evidence, implemented, testing | implemented spec | — |
| [SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md](SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md) | Implemented MVP | testing | evidence, implemented, testing | implemented spec | — |
| [SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md](SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md) | Deferred / To-Spec | general | deferred | deferred spec | — |
| [SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md](SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Implemented MVP (Phase 150) | testing | evidence, implemented, testing | implemented spec | [PLAN-150](../plan/PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md); SPEC-087; TASK-1805 |
| [SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md](SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) | Implemented MVP (Phase 151; Phase 176 recursive cleanup) — recursive bounded generation remains fail-closed/deferred | testing | core-ir, evidence, implemented, semantics, testing | implemented spec with deferred tail | [PLAN-151](../plan/PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md); [PLAN-176](../plan/PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md); TASK-1805 |
| [SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md](SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md) | Implemented MVP (Phase 152) | effect-system | authority, effect-system, implemented | implemented spec | [PLAN-152](../plan/PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md); TASK-1805 |
| [SPEC-089-LIST-BUILTIN-TO-STDLIB.md](SPEC-089-LIST-BUILTIN-TO-STDLIB.md) | Implemented MVP (Phase 153; Phase 176 removed legacy `Value::List`) | general | implemented, runtime, surface | implemented spec | [PLAN-153](../plan/PLAN-153-LIST-BUILTIN-TO-STDLIB.md); [PLAN-176](../plan/PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md); TASK-1805 |
| [SPEC-090-TYPE-ANNOTATION-QUIRKS.md](SPEC-090-TYPE-ANNOTATION-QUIRKS.md) | Implemented MVP (Phase 154) | type-system | diagnostics, implemented, references, type-system | implemented spec | [PLAN-154](../plan/PLAN-154-TYPE-ANNOTATION-QUIRKS.md) |
| [SPEC-091-LET-DESTRUCTORS.md](SPEC-091-LET-DESTRUCTORS.md) | Implemented MVP (Phase 155) | language-surface | grammar, implemented, surface | implemented spec | [PLAN-155](../plan/PLAN-155-LET-DESTRUCTORS.md); TASK-1805 |
| [SPEC-092-PARSER-BLOCKER-RESOLUTION.md](SPEC-092-PARSER-BLOCKER-RESOLUTION.md) | Implemented MVP (Phase 156) | language-surface | grammar, implemented, surface | implemented spec | [PLAN-156](../plan/PLAN-156-PARSER-BLOCKER-RESOLUTION.md); TASK-1805 |
| [SPEC-094-LANGUAGE-SURFACE-FIX.md](SPEC-094-LANGUAGE-SURFACE-FIX.md) | Implemented MVP (Phase 158; Phase 176 completed deferred tail) | language-surface | grammar, implemented, surface | implemented spec | [PLAN-158](../plan/PLAN-158-LANGUAGE-SURFACE-FIXES.md); [PLAN-176](../plan/PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md); TASK-1805 |
| [SPEC-095-ASH-SURFACE-GRAMMAR.md](SPEC-095-ASH-SURFACE-GRAMMAR.md) | draft | language-surface | grammar, surface | normative spec | — |
| [SPEC-095a-CURRENT-GRAMMAR.md](SPEC-095a-CURRENT-GRAMMAR.md) | active | language-surface | current-state, grammar, surface | current-state spec | — |
| [SPEC-095b-TARGET-GRAMMAR.md](SPEC-095b-TARGET-GRAMMAR.md) | draft | language-surface | grammar, surface, target-state | target-state spec | SPEC-095c; NOTE-015; NOTE-021; parser plans/tasks |
| [SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md) | draft | language-surface | grammar, syntax, semantics, target-state, tooling | target-state spec | SPEC-095b; SPEC-097b; SPEC-098c; PLAN-167; PLAN-171; PLAN-172; PLAN-173 |
| [SPEC-096-UNIFIED-EFFECT-SYSTEM.md](SPEC-096-UNIFIED-EFFECT-SYSTEM.md) | Historical bridge draft superseded by SPEC-096a/SPEC-096b split | contracts | authority, contract, effect-system, references | superseded/historical spec | SPEC-096a; SPEC-096b; PLAN-181 |
| [SPEC-096a-CURRENT-EFFECT-SYSTEM.md](SPEC-096a-CURRENT-EFFECT-SYSTEM.md) | active | effect-system | authority, current-state, effect-system | current-state spec | — |
| [SPEC-096b-TARGET-EFFECT-SYSTEM.md](SPEC-096b-TARGET-EFFECT-SYSTEM.md) | draft | effect-system | authority, effect-system, target-state | target-state spec | NOTE-013; NOTE-020; NOTE-022; NOTE-034; NOTE-035 |
| [SPEC-097-TYPE-SYSTEM-CHANGES.md](SPEC-097-TYPE-SYSTEM-CHANGES.md) | Historical bridge draft superseded by SPEC-097b and SPEC-100 for target typing | effect-system | authority, effect-system, references, type-system | superseded/historical spec | SPEC-097b; SPEC-100; PLAN-181 |
| [SPEC-097a-CURRENT-TYPE-SYSTEM.md](SPEC-097a-CURRENT-TYPE-SYSTEM.md) | active | type-system | current-state, type-system | current-state spec | — |
| [SPEC-097b-TARGET-TYPE-SYSTEM.md](SPEC-097b-TARGET-TYPE-SYSTEM.md) | draft | type-system | target-state, type-system | target-state spec | NOTE-025; NOTE-031; NOTE-033; SPEC-095c; SPEC-098c; SPEC-100; PLAN-173 |
| [SPEC-098-IR-CHANGES.md](SPEC-098-IR-CHANGES.md) | Historical bridge draft superseded by SPEC-098b/SPEC-098c for target IR/lowering | core-ir | authority, core-ir, effect-system, references, semantics | superseded/historical spec | SPEC-098b; SPEC-098c; PLAN-181 |
| [SPEC-098a-CURRENT-IR.md](SPEC-098a-CURRENT-IR.md) | active | core-ir | core-ir, current-state, semantics | current-state spec | — |
| [SPEC-098b-TARGET-IR.md](SPEC-098b-TARGET-IR.md) | draft | core-ir | core-ir, semantics, target-state | target-state spec | SPEC-098c; SPEC-099b; SPEC-100; PLAN-165 |
| [SPEC-098c-SURFACE-TO-CORE-LOWERING.md](SPEC-098c-SURFACE-TO-CORE-LOWERING.md) | draft | core-ir | core-ir, grammar, semantics, target-state | target-state spec | SPEC-095c; SPEC-097b; SPEC-098b; SPEC-100; PLAN-167; PLAN-171; PLAN-172; PLAN-173 |
| [SPEC-099-CORE-LANGUAGE.md](SPEC-099-CORE-LANGUAGE.md) | Implemented MVP (Phase 161) / design-level formal spec | core-ir | core-ir, implemented, semantics | implemented spec / formal design reference | SPEC-098b; SPEC-100; SPEC-101; SPEC-102; [PLAN-161](../plan/PLAN-161-CORE-ASH-IR-FOUNDATION.md); TASK-1805 |
| [SPEC-099-OPERATIONAL-SEMANTICS.md](SPEC-099-OPERATIONAL-SEMANTICS.md) | Historical bridge draft superseded by SPEC-099b for target operational semantics | core-ir | authority, core-ir, effect-system, references, semantics | superseded/historical spec | SPEC-099b; PLAN-181 |
| [SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md](SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md) | active | core-ir | core-ir, current-state, semantics | current-state spec | — |
| [SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) | draft | core-ir | core-ir, runtime, semantics, target-state | target-state spec | SPEC-098b; SPEC-098c; SPEC-100; SPEC-101; PLAN-167 |
| [SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md](SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) | draft | core-ir | core-ir, semantics | normative spec | — |
| [SPEC-100-CORE-TYPE-CHECKING.md](SPEC-100-CORE-TYPE-CHECKING.md) | Implemented MVP (Phase 162) / design-level formal spec | core-ir | core-ir, implemented, semantics, type-system | implemented spec / formal design reference | SPEC-097b; SPEC-098b; SPEC-099; [PLAN-162](../plan/PLAN-162-CORE-ASH-TYPE-CHECKING.md); PLAN-165; TASK-1805 |
| [SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md](SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md) | Implemented MVP (Phase 163); force-time authority phrased in provider/handler/resource target terms | core-ir | authority, core-ir, implemented, semantics, type-system | implemented spec | NOTE-028; SPEC-096b; SPEC-099; SPEC-100; [PLAN-163](../plan/PLAN-163-CORE-LAZY-MEMO-MODES.md); TASK-1805 |
| [SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md](SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md) | Implemented MVP (Phase 164) | core-ir | core-ir, implemented, semantics | implemented spec | SPEC-099c; [PLAN-164](../plan/PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md); TASK-1805 |
| [SPEC-BUILTIN-FN.md](SPEC-BUILTIN-FN.md) | Draft | general | orientation | normative spec | — |
