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
| SPEC-047 | Act Monad | Draft | Historical reference for the `Act<A>` carrier and `act {}` blocks; current productive docs should prefer checked helper/profile examples and target-current migration notes. |
| SPEC-048 | Proc Library | Draft | Historical reference for the `Proc<A>` carrier with async process handles (`P<A>`); current productive docs should prefer checked `std::process` helper examples. |
| SPEC-049 | Process Runtime Semantics | Draft | Runtime semantics for process identities, affine/linear handles, child environment projection, `yield`, async `par`, `await`, `join`, and `gather` |
| SPEC-050 | Operational Bottom and Scoped Handling | Draft | Operational failure as tower/entity-indexed bottom, `fail`, `with_error`, process-observation failure, aggregation, and workflow-boundary reinterpretation hooks |
| SPEC-051 | Workflow Semantics | Draft | Workflow as governance above `Proc`: admission, roles/capabilities, `requires`/`ensures`, obligations, reporting, `WorkflowFailure`, and lower-failure reinterpretation |
| [SPEC-052](SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md) | Capability Interfaces and Implementations | Draft | Stateless capability interfaces, Ash-defined implementation recipes, binding-time selection, module visibility, conformance, derived implementations, and Phase 104 runtime API invocation pilots |
| [SPEC-053](SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md) | Runtime Resources and Authority Provenance | Draft | Resource types, resource instances, resource bindings, host/internal/derived authority provenance, lifecycle, split/join policy, resource-backed operation evidence, and Phase 104 internal-resource pilots |
| [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) | Generalized Typed Do-Notation | Implemented MVP | Explicit `do:K` computation blocks, selected `Monad<K>` evidence for implemented Act/Proc/Workflow and Phase 133 Option/Result targets, typed elaboration, Act migration, tower/failure behavior, and diagnostics; target inference, guards, full List Monad, and arbitrary user Monad execution remain follow-up work. |
| [SPEC-055](SPEC-055-MONAD-COMPREHENSION-SYNTAX.md) | Monad Comprehension Syntax | Implemented MVP | Explicit-target bracket comprehension syntax `[result | qualifiers]: K` as a container-view spelling of generalized typed do-notation; parser-surface fidelity, typed-do elaboration reuse, tower rules, diagnostics, and Phase 133 Option/Result evidence are implemented, while inference, guards, full List Monad semantics, and arbitrary user Monad execution remain follow-up work. |
| [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) | First-Class Workflow Carrier | Implemented MVP | Historical `Workflow<A>` MVP as a contract-indexed `Proc<A>` carrier; its `WorkflowForm`/projection language is now superseded for target planning by ambient computation rows, Core/CPS carriers, trace/monitor sidecars, obligations, evidence, and provenance. Current work should treat source-ordered `WorkflowHeaderEvent`s, compiler-known `workflow::...` builtins, workflow operations, legacy-compatible `requires:`/`ensures:` syntax, deprecated workflow declaration translation, `do:Workflow`, and `[...]: Workflow` comprehension integration as historical/removal context unless a Phase 201 audit explicitly maps the fact to a target carrier. |
| [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md) | Unified Type/Module Pipeline and Semantic Summaries | Implemented MVP | Tier 0 substrate for DESIGN-034: ordinary `type` declarations flow through ModuleFile, core semantic summaries, engine transport, and TypeEnv registration while source-snippet type discovery is replaced or fenced. Later type-expression IR, sealed domains, type functions, normalization, associated families, and propositions remain deferred. |
| [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) | Canonical Type-Expression IR, Projection Identities, and Kind/Arity Substrate | Implemented MVP | DESIGN-034 SPEC-B: internal type-expression substrate promoting canonical computation-grade identities, canonical projection IR, rigid/neutral carriers, and explicit kind/arity validation while keeping `type fn`, sealed domains, normalization, computation-summary export/import, and new public syntax deferred. |
| [SPEC-059](SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md) | Sealed Type-Level Domains | Implemented MVP | DESIGN-034 SPEC-C: implements nominal sealed type-level domains, marker-constructor identities, domain-kind metadata, and visibility-aware public domain-summary transport while deferring normalization, direct structural `type fn`, associated type families, and promoted data constructors. |
| [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md) | Normalizer and Definitional Equality Core | Implemented MVP | DESIGN-034 SPEC-D: implements the internal total normalizer, canonical normal forms, fixture equation tables, normalize-and-compare definitional equality, neutrality diagnostics, and narrow TypeEnv forcing-point adoption while deferring public `type fn`, associated families, equation export/import, and proof search. |
| [SPEC-061](SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) | Direct Structural Type Functions | Implemented MVP | DESIGN-034 SPEC-E: first user-facing type-computation surface, adding module-local `type fn` declarations over sealed domains with checked equations, source/result-domain validation, ordered residual coverage/overlap, structural recursion validation, normalizer integration, diagnostics, and public computation-head leakage rejection while deferring equation export/import, associated families, and proof search. |
| [SPEC-062](SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md) | Module-Summary Export/Import for Type Computation | Implemented MVP | DESIGN-034 SPEC-F: implemented the public module-summary boundary for type computation, exporting/importing public sealed-domain and transparent public `type fn` summaries through core-owned semantic summaries while preserving private equation opacity, import-order independence, summary versioning, dependency-closure helper opacity, and normalizer non-inversion. |
| [SPEC-063](SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md) | Associated Type-Family Computation | Implemented MVP | DESIGN-034 SPEC-G: implements sealed associated type-family computation over the total type-computation substrate, preserving SPEC-035 simple substitution while adding explicit family projections, unique generic impl-family reduction, rigid where-bound projections, recursive family totality, V4 public family summaries/imports, diagnostics, acceptance evidence, and non-inversion boundaries. |
| [SPEC-064](SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md) | Constraint and Proposition Layer | Implemented MVP | DESIGN-034 SPEC-H: implemented conservative type-level proposition layer over normalized types, adding canonical equality/disequality/interface-bound/named-predicate proposition carriers, TypeEnv obligation generation, no-inversion solver outcomes, V5 public proposition summaries, diagnostics, and acceptance/non-interference evidence while deferring unrestricted SMT/proof search and type-function inversion. |
| [SPEC-065](SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md) | Promoted Data Constructors and Named Data Kinds | Implemented MVP | DESIGN-034 SPEC-I: implements opt-in promoted data-kind and constructor identities, V6 summary transport, TypeEnv registration/kinding, promoted-app normalizer/proposition integration, hidden selected-summary dependency metadata, and non-interference with runtime ADTs/sealed-domain markers. Source `data kind` declarations parse; source-to-summary lowering remains outside this MVP. |
| [SPEC-066](SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) | Type Holes and Partial Type-Constructor Application | Implemented MVP | Explicit source `_` holes in audited do-target type-argument positions and partial constructor application carriers/elaboration such as `Result<_, E>`, including kind/arity/ambiguity diagnostics and do-target shape elaboration without implicit currying, Monad evidence, HKT binders, or inversion. |
| [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) | Constructor-Kinded Parameters and HKT | Implemented MVP | Kinded binders such as `M : * -> *`, constructor-variable application, higher-kinded interface/impl evidence shape and overlap rejection, explicit `Monad<K>` do-target evidence lookup, and summary non-interference are implemented. Runtime lowering through arbitrary user-defined Monad methods, law proving, automatic do-target inference, unrestricted type lambdas, higher-rank polymorphism, and broad multi-parameter constructor classes remain deferred. |
| [SPEC-068](SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md) | Pattern and Exhaustiveness Canonicalization | Implemented MVP | Audit-first rollout of transparent alias and selected reducible projection canonicalization into ordinary ADT pattern checking and exhaustiveness without neutral-head inversion, constructor-name leakage, GADT/refinement patterns, type-level runtime matching, broad equality adoption, or ADT layout changes. |
| [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md) | Alpha Visible Tower Algebra and Generalized Do Lowering | Implemented MVP | DESIGN-040 alpha target for visible `Act`/`Proc`/`Workflow`/user computation algebra, full `Monad<K>` evidence-selected `do:K` bind lowering, typed computation-expression/TCIR boundary, AMIR/bytecode traceability, OODA demotion to library/template/lint surface, `do:Result` operational-bottom execution evidence, and run/daemon artifact equivalence at the alpha checked workflow-boundary carrier. |
| [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md) | Alpha Runtime Kernel and OS-Facing Execution Surface | Implemented MVP | DESIGN-041 alpha runtime regime: one `RuntimeKernel` with one-shot `ash run` and local `ashd` host modes, explicit roots, definition/instance/artifact identity, admission authority, daemon start records, policy-profile grant enforcement, daemon child-failure traces, reload semantics, and same verified language artifact summary across host lifetimes at the alpha checked workflow-boundary carrier. |
| [SPEC-071](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md) | Reference Corpus Metadata and Maintenance | Implemented MVP | Phase 124 implemented the pilot top-level `reference/` corpus, SPEC-071 frontmatter-bearing authority/methodology/style/status pages, Pure/Act/Proc/Workflow pages, agent cards, example/status classifications, and R71 drift/evidence closeout while preserving `docs/` as working and historical documentation. |
| [SPEC-072](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md) | Tower Callable Type and Closure Syntax | Implemented MVP | Implements the first tower-callable syntax slice: target pure callable type spelling `(A, B) -> C`, removed-form diagnostics for the historical named callable spelling, exact callable arity, pure closure shorthand `|args| -> body`, fail-closed reserved Act/Proc/Workflow callable and closure arrows, and std/reference migration evidence while deferring higher-stratum callable application semantics. |
| [SPEC-073](SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md) | Ashgrove Install, Update, Cleanup, and Git Deployment | Implemented MVP | Defines `ashgrove <command>` as the user-local Ash toolchain/deployment manager: source and binary tarball installs, immutable XDG-local toolchain bundles containing `ash`, `ashgrove`, stdlib, runtime-support metadata, selected standard tooling, update/default/remove/cleanup policy, explicit-digest tarball URL updates, mandatory fail-closed trust/signing enforcement at implemented release/download/git boundaries, and lower-case `ash.toml` git URL + tag/rev dependencies resolved into exact commits in `ash.lock` before hosted registry package management. Non-goals remain explicit: no hosted registry service, global/system roots, OS package-manager integration, arbitrary SemVer solver, or signed release-index-as-digest resolver. |
| [SPEC-074](SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) | Ashgrove Source Payload and Local-State Ignore Policy | Accepted/Implemented | Amends SPEC-073 source-root install semantics by separating reproducible source payload from local checkout state: gitignored and known local-state paths such as `.agents/`, nested `target/`, worktrees, and caches do not affect source-root payload digest or isolated build copy, while nonignored source payload changes remain fail-closed and source-archive attestation behavior remains separate. |
| [SPEC-075](SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md) | Reference Slice 2 Runtime, Toolchain, and Maintenance Manual | Implemented MVP | Expands the reference corpus with subsystem detail pages, reader-journey basics, maintenance metadata/staleness procedures, Ashgrove/RuntimeKernel/stdlib reference pages, status surfaces, alpha limitations, feature/drift/verification evidence, agent cards, and a path-based Slice 2 staleness audit while preserving SPEC-071 governance. |
| [SPEC-076](SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md) | Explicit Refutable Matching and Exhaustiveness | Implemented MVP | Bans implicit refutable matching by requiring irrefutable binder patterns, exhaustive eliminator arm sets or blocked diagnostics, structured matching diagnostics, `if let ... else` as total by implicit complement, and current selective `receive` as an explicit refutable filtering form; runtime pattern errors remain defensive for unchecked IR/host-created values. |
| [SPEC-077](SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md) | Ash Test Runner Synthesized and Small-World Completion | Implemented MVP | Implements the bounded DESIGN-022/DESIGN-023 MVP after Phase 76B: live checked/lowered runner snapshots from ordinary CLI files, supported synthesized contract/policy/obligation execution, deterministic small-world target execution, richer finite domains, CLI integration, repro artifacts, and broad verification while leaving open-domain/arbitrary runtime semantics deferred. |
| [SPEC-078](SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md) | Standard Algebra Library and Monad Remediation | Implemented MVP | Adds the `std::algebra` namespace with source-visible Semigroup, Monoid, Functor, Applicative, and Monad interfaces; reconciles stdlib/prelude evidence for Option, Result, List, Act, Proc, Workflow, do-notation, and comprehensions; and splits law proof/test derivation into a generated-test follow-up. |
| [SPEC-079](SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md) | Standard Algebra Comonad and Kleisli Helper Surfaces | Implemented MVP | Adds the current `std::algebra::comonad::Comonad` interface, records generated-law handoff ownership, and explicitly defers generic Kleisli/Cokleisli helpers, Coapplicative, `std::category`, broad category abstractions, and unsound partial/opaque Comonad instances. |
| [SPEC-080](SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md) | Interface Evidence Constraints | Implemented MVP | Adds interface-level evidence constraints such as `interface Monad<M : * -> *> where M: Applicative`, making `M: Monad` entail `M: Applicative` and related standard algebra constraints such as `Applicative -> Functor` and `Monoid -> Semigroup` through typechecker-verified required evidence without object-hierarchy wording, automatic derivation, or blanket impl synthesis; final algebra interfaces use generic payload method signatures rather than monomorphic `Int` placeholders. |
| [SPEC-081](SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md) | Law Test Evidence Substrate | Implemented MVP | Implements fail-closed empirical `by test` evidence modes for authored/manual tests, law-as-property execution, and finite small-world enumeration, all runnable from the shipped `ash` binary without Cargo or Rust tooling while reserving symbolic/solver proofs for future non-test evidence families. |
| [SPEC-082](SPEC-082-PROPERTY-GENERATION-AND-SHRINKING-SUBSTRATE.md) | Property Generation and Shrinking Substrate | Implemented MVP | Implements generator, binding, counterexample, and shrinking substrate for `ash test` property evidence, with broader generator/shrinker expansion left as follow-up work. |
| [SPEC-083](SPEC-083-LAW-COVERAGE-AND-MUTATION-TESTING.md) | Law Coverage and Mutation Testing | Implemented MVP | Adds law/test coverage reporting and bounded mutation testing for Ash tests/laws. |
| [SPEC-084](SPEC-084-FLAKY-TEST-QUARANTINE-AND-DISTRIBUTED-ORCHESTRATION.md) | Flaky-Test Quarantine and Distributed Orchestration | Implemented MVP | Adds retry/flake classification, quarantine metadata, shard planning, local shard execution, and result merging. |
| [SPEC-085](SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md) | Proof-Producing Synthesis Todo Spec | Deferred / To-Spec | Documents future proof-producing synthesis as a deferred non-test proof evidence family. |
| [SPEC-086](SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | QuickCheck Arbitrary and Strategy Property Testing | Implemented MVP | Adds the `test::quickcheck` standard-library property-testing substrate with `Strategy<T>`, `Arbitrary<T>`, compositional strategy overrides, law/property enforcement distinction, and evidence-cache schema groundwork; hardened by SPEC-087. |
| [SPEC-087](SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) | QuickCheck v1 Ordinary Strategy Semantics | Implemented MVP with deferred tail | Hardens Phase 150 QuickCheck from metadata/runner bridge MVP into ordinary pure `Strategy<A>` values, helper-first `GenContext`, in-scope `Arbitrary<A>` evidence, pure strategy overrides, stable RNG/split, explicit shrink semantics, random seed/replay policy, and aggregate empirical evidence history. Phase 176 keeps recursive API/config visible while recursive bounded generation remains fail-closed/deferred. |
| [SPEC-101](SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md) | Lazy and Memo Computation Modes | Implemented MVP | Defines and implements Core-level `lazy` and `memo` mode carriers, force semantics, row accounting, runtime memo behavior, tracing expectations, and lowering through existing CPS IR forms without adding new CPS term variants. |

## Deprecated Specifications

None currently.

## Current vs Target State Specifications

The following specifications are split into **current state** (what the compiler/runtime
do today) and **target state** (where we want the language to evolve):

| Layer | Current State | Target State | Description |
|-------|---------------|--------------|-------------|
| Grammar | [SPEC-095a](SPEC-095a-CURRENT-GRAMMAR.md) | [SPEC-095b](SPEC-095b-TARGET-GRAMMAR.md) | Surface syntax |
| Effect System | [SPEC-096a](SPEC-096a-CURRENT-EFFECT-SYSTEM.md) | [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md) | Effect accounting |
| Type System | [SPEC-097a](SPEC-097a-CURRENT-TYPE-SYSTEM.md) | [SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md) | Type checking |
| IR | [SPEC-098a](SPEC-098a-CURRENT-IR.md) | [SPEC-098b](SPEC-098b-TARGET-IR.md) | Intermediate representation |
| Operational Semantics | [SPEC-099a](SPEC-099a-CURRENT-OPERATIONAL-SEMANTICS.md) | [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) | Runtime semantics |

The current-state specs are **frozen** against live code and serve as implementation
authorities. The target-state specs are **living documents** that evolve with design
decisions and are refined as implementation progresses.

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
