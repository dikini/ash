# Ash Specification Index

This directory contains the canonical specifications for the Ash workflow language.

## Active Specifications

| Spec | Title | Status | Description |
|------|-------|--------|-------------|
| SPEC-001 | Intermediate Representation | Active | Core AST types, serialization, and IR semantics |
| SPEC-002 | Surface Language | Active | Surface syntax and parsing |
| SPEC-003 | Type System | Active | Type checking, inference, and constraint solving |
| SPEC-004 | Operational Semantics | Active | Operational semantics and evaluation rules |
| SPEC-005 | Ash CLI Specification | Active | Command-line interface and commands |
| SPEC-006 | Policy Definition Syntax | Active | Policy definition and structure |
| SPEC-007 | Policy Combinators | Active | Policy combination operators |
| SPEC-008 | Dynamic Policies | Active | Runtime policy modification |
| SPEC-009 | Module System | Active | Namespaces, imports, and module resolution |
| SPEC-010 | Embedding | Active | Embedding Ash in host applications |
| SPEC-011 | REPL | Active | Interactive REPL semantics |
| SPEC-012 | Imports | Active | Import system and resolution |
| SPEC-013 | Streams | Active | Stream processing and semantics |
| SPEC-014 | Behaviours | Active | Behaviour definitions and contracts |
| SPEC-015 | Typed Providers | Active | Capability providers with types |
| SPEC-016 | Output | Active | Output formatting and destinations |
| SPEC-017 | Capability Integration | Active | Capability integration with system features |
| SPEC-018 | Capability Matrix | Active | Capability permission matrix |
| SPEC-019 | Role Runtime Semantics | Active | Role-based execution semantics |
| SPEC-020 | Algebraic Data Types | Active | Sum types, product types, and pattern matching |
| SPEC-046 | Lean Reference | Active | Reference Lean formalization |
| SPEC-021 | Runtime Observable Behavior | Active | Runtime behavior observation |
| SPEC-022 | Workflow Typing with Constraints | Active | Contracts, obligations, and linear resource tracking |
| SPEC-023 | Proxy Workflows | Active | Proxy workflow patterns and semantics |
| SPEC-025 | Small-Step Operational Semantics | Active | Workflow-first small-step semantics and state taxonomy |
| SPEC-026 | Implementation Conformance Contract | Active | Cross-implementation conformance surfaces, bounded nondeterminism, and comparison rules |
| SPEC-027 | Pure Functions | Draft | fn construct for pure computation, match/if expressions, purity enforcement |
| SPEC-028 | Function Constraint System | Draft | fn contract vocabulary, constraint evolution path, Z3 integration plan |
| SPEC-045 | Ash Wiki Knowledge Substrate | Draft | Static-first metadata, authority, supersession, audit, and human/AI service contract over the project corpus |
| SPEC-047 | Act Monad | Draft | First-class effectful computation, Act<A> type constructor, act {} blocks, invoke/unit/bind builtins, unifying pure expressions and effectful workflows |
| SPEC-048 | Proc Library | Draft | Minimal process-structured computation type/library (`Proc<A>`) with async process handles (`P<A>`), library-first process combinators, and deferred runtime-heavy features |
| SPEC-049 | Process Runtime Semantics | Draft | Runtime semantics for process identities, affine/linear handles, child environment projection, `yield`, async `par`, `await`, `join`, and `gather` |
| SPEC-050 | Operational Bottom and Scoped Handling | Draft | Operational failure as tower/entity-indexed bottom, `fail`, `with_error`, process-observation failure, aggregation, and workflow-boundary reinterpretation hooks |
| SPEC-051 | Workflow Semantics | Draft | Workflow as governance above `Proc`: admission, roles/capabilities, `requires`/`ensures`, obligations, reporting, `WorkflowFailure`, and lower-failure reinterpretation |
| [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) | Capability Interfaces and Implementations | Draft | Stateless capability interfaces, Ash-defined implementation recipes, binding-time selection, module visibility, conformance, derived implementations, and Phase 104 runtime API invocation pilots |
| [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) | Runtime Resources and Authority Provenance | Draft | Resource types, resource instances, resource bindings, host/internal/derived authority provenance, lifecycle, split/join policy, resource-backed operation evidence, and Phase 104 internal-resource pilots |
| [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) | Generalized Typed Do-Notation | Implemented MVP | Explicit `do:K` computation blocks, MVP Act/Proc Monad-shaped dictionaries, typed elaboration, Act migration, tower/failure behavior, and diagnostics |
| [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) | Monad Comprehension Syntax | Implemented MVP | Explicit-target bracket comprehension syntax `[result | qualifiers]: K` as a container-view spelling of generalized typed do-notation for MVP Act/Proc targets; parser-surface fidelity, typed-do elaboration reuse, tower rules, and diagnostics are implemented, while inference, guards, and pure List/Option/Result dictionaries are deferred |
| [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) | First-Class Workflow Carrier | Implemented MVP | `Workflow<A>` as a contract-indexed `Proc<A>` carrier with a blocking `WorkflowForm`/projection/obligation semantic gate, source-ordered legacy `WorkflowHeaderEvent`s, non-denotable contract argument classes, qualified compiler-known `workflow::...` builtins, WorkflowForm-aware algebra/call elaboration for all first-slice workflow operations, coverage/evidence relations, legacy-compatible `requires:`/`ensures:` contract syntax, deprecated legacy workflow declaration translation to the same semantics, delayed lower Proc/Act coverage obligations, `do:Workflow`, and `[...]: Workflow` comprehension integration |
| [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) | Unified Type/Module Pipeline and Semantic Summaries | Implemented MVP | Tier 0 substrate for DESIGN-034: ordinary `type` declarations flow through ModuleFile, core semantic summaries, engine transport, and TypeEnv registration while source-snippet type discovery is replaced or fenced. Later type-expression IR, sealed domains, type functions, normalization, associated families, and propositions remain deferred. |
| [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) | Canonical Type-Expression IR, Projection Identities, and Kind/Arity Substrate | Implemented MVP | DESIGN-034 SPEC-B: internal type-expression substrate promoting canonical computation-grade identities, canonical projection IR, rigid/neutral carriers, and explicit kind/arity validation while keeping `type fn`, sealed domains, normalization, computation-summary export/import, and new public syntax deferred. |
| [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md) | Sealed Type-Level Domains | Implemented MVP | DESIGN-034 SPEC-C: implements nominal sealed type-level domains, marker-constructor identities, domain-kind metadata, and visibility-aware public domain-summary transport while deferring normalization, direct structural `type fn`, associated type families, and promoted data constructors. |
| [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md) | Normalizer and Definitional Equality Core | Implemented MVP | DESIGN-034 SPEC-D: implements the internal total normalizer, canonical normal forms, fixture equation tables, normalize-and-compare definitional equality, neutrality diagnostics, and narrow TypeEnv forcing-point adoption while deferring public `type fn`, associated families, equation export/import, and proof search. |
| [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) | Direct Structural Type Functions | Draft | DESIGN-034 SPEC-E: planned first user-facing type-computation surface, adding module-local `type fn` declarations over sealed domains with checked equations, source/result-domain validation, ordered residual coverage/overlap, structural recursion validation, normalizer integration, and diagnostics while rejecting public computation-head leakage and deferring equation export/import, associated families, and proof search. |

## Deprecated Specifications

None currently.

## Specification Template

New specifications should follow this structure:

```markdown
# SPEC-XXX: Title

**Status:** Active | Draft | Deprecated
**Supersedes:** SPEC-YYY (if applicable)
**Related:** SPEC-ZZZ, SPEC-WWW

## Summary

Brief description of what this specification defines.

## Motivation

Why this specification exists and what problems it solves.

## Specification

### Section 1

Detailed technical content.

## Implementation Tasks

- TASK-###: Description

## Changelog

### YYYY-MM-DD

- Initial version
```

## Cross-Reference Guide

When writing specifications, use these cross-reference formats:

- Type system rules: `See [SPEC-003-TYPE-SYSTEM](SPEC-003-TYPE-SYSTEM.md)`
- IR constructs: `See [SPEC-001-IR](SPEC-001-IR.md)`
- Workflow contracts: `See [SPEC-022-WORKFLOW-TYPING](SPEC-022-WORKFLOW-TYPING.md)`

## Review Process

1. Specifications start as drafts in `todo-examples/definitions/`
2. Once implemented and tested, moved to `docs/spec/`
3. Marked as Active when complete
4. Updated via patch commits with changelog entries
