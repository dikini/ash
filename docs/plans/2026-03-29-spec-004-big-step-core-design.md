# SPEC-004 Big-Step Core Semantics Design

**Date:** 2026-03-29

## Goal

Revise [docs/spec/SPEC-004-SEMANTICS.md](../spec/SPEC-004-SEMANTICS.md) into a complete, proof-suitable big-step core semantics for Ash. The revised document should support Lean formalization and later small-step refinement without sacrificing the runtime-facing authority and execution-neutrality already established by the current spec set.

## Problem Statement

The current [SPEC-004](../spec/SPEC-004-SEMANTICS.md) is a strong runtime-oriented draft, but it is not yet ideal as a proof corpus document.

Current proof blockers:

- expression semantics are split between informal `eval(...)` use and scattered formal rules;
- pattern semantics are split between `bind(...)` helpers and workflow-form prose;
- rejection propagation and failure ownership are partly implied rather than judgment-shaped;
- helper relations such as `select_receive_outcome` and `combine_parallel_outcomes` are semantically significant but not fully contract-shaped;
- deterministic and nondeterministic fragments are not separated clearly enough for theorem statements.

These gaps force a formalizer to infer judgment boundaries that should instead be stated directly in the spec.

## Design Constraints

This design must preserve the following project constraints:

- [docs/reference/formalization-boundary.md](../reference/formalization-boundary.md) already treats `SPEC-004` as part of the canonical semantic corpus.
- [docs/spec/SPEC-001-IR.md](../spec/SPEC-001-IR.md) defines the canonical core forms and execution-neutral IR boundary that `SPEC-004` must continue to respect.
- Runtime authority remains normative: the runtime owns validation, rejection, commitment, trace, and provenance.
- The design should be Lean-first for the pure/core fragment, while still preserving the full runtime-bearing contract as abstract helper relations.
- The result must remain suitable for future Rust convergence and eventual small-step derivation.

## Chosen Approach

Use a layered semantics structure:

1. **Proof-shaped semantic core** for pure expressions, patterns, and deterministic workflow fragments.
2. **Explicit runtime-helper contracts** for mailbox selection, policy lookup/evaluation, action execution, and parallel outcome aggregation.
3. **Meta-property sections** for determinism scope, admissible nondeterminism, and semantic invariants.

This is preferred over a single full-runtime proof pass because it makes the pure fragment tractable without prematurely over-specifying provider internals.

## Semantic Architecture

### 1. Semantic Domains and Algebra

Strengthen front-matter definitions so every later rule relies on declared operators and domains rather than inferred meaning.

Required additions or normalizations:

- total effect order and join operator laws;
- trace concatenation operator and order preservation intent;
- environment extension and shadowing rules;
- explicit runtime failure categories;
- distinction between pure judgments and runtime-bearing workflow judgments.

### 2. Core Judgments

The revised document should define, near the front, the full set of judgments used later:

- workflow big-step judgment:
  - `Γ, C, P, Ω, π ⊢ w ⇓ out`
- expression evaluation judgment:
  - `Γ ⊢ e ⇓expr v`
- pattern matching judgment:
  - `match(p, v) ⇓ ΔΓ`
  - `match(p, v) ⇓ fail`
- optional guard judgment if the final wording benefits from separating guards from expressions.

The main rule is: no rule may rely on an undefined helper such as `eval(...)` or `bind(...)` after the revision.

### 3. Propagation and Runtime Conventions

A dedicated conventions section should own:

- left-to-right premise evaluation;
- rejection propagation;
- trace-prefix preservation on failure;
- effect accumulation through failing prefixes;
- obligation and provenance state at the rejection point;
- lookup-failure mapping to `RuntimeFailure(...)`.

This keeps proof obligations out of duplicated prose attached to individual rules.

### 4. Pure Core Rules

The pure fragment should be made directly proof-shaped.

Required rule families:

- literals;
- variables;
- list / record / variant formation;
- field access;
- constructor evaluation;
- match expression evaluation;
- equality and boolean connectives;
- pattern rules for wildcard, variable, literal, tuple/list, record, and variant patterns.

This fragment should be deterministic by construction.

### 5. Runtime-Bearing Workflow Rules

Workflow rules should continue to model the full Ash runtime contract, but helper-backed behavior must be isolated behind explicit contracts.

Core workflow rules to normalize:

- `OBSERVE`
- `RECEIVE`
- `ORIENT`
- `PROPOSE`
- `DECIDE`
- `CHECK`
- `ACT`
- `LET`
- `IF`
- `SEQ`
- `PAR`

The revision should not erase runtime authority. Instead, it should make the runtime boundary precise enough to cite in proofs and implementation alignment.

## Helper Contract Strategy

Semantically significant helpers remain abstract, but each must become contract-shaped. For every helper relation, the revised spec should state:

- input domain;
- output domain;
- whether it is deterministic;
- whether it is partial;
- how failures map to `Reject(...)`;
- any preserved laws needed by the surrounding rules.

Priority helpers:

- `lookup(C, cap)`
- `lookup(P, policy)`
- `select_receive_outcome(...)`
- `perform_action(...)`
- `check_obligation(...)`
- `combine_parallel_outcomes(...)`
- provenance merge/join helpers.

## Determinism Boundary

The revised spec should explicitly classify:

### Deterministic fragment

- expression evaluation;
- pattern matching;
- `LET` once expression and pattern judgments are explicit;
- `IF` when the condition fragment is deterministic;
- `SEQ` over deterministic subworkflows;
- closed pure/core workflow fragments without `RECEIVE`, `PAR`, or provider-dependent helpers.

### Nondeterministic fragment

- `RECEIVE` due to source selection and scheduling contracts;
- `PAR` due to branch interleaving and runtime completion order;
- provider/runtime-dependent helper relations where the spec intentionally defines an admissible result set rather than one concrete computation path.

This split should be explicit so proofs do not accidentally overclaim global determinism.

## Proof Targets Enabled by the Revision

The design aims to make the following theorem families natural to state:

- expression determinism;
- pattern determinism;
- constructor purity;
- determinism of the closed pure workflow fragment;
- monotonicity of effect accumulation;
- trace-order preservation;
- provenance-lineage preservation;
- adequacy of helper contracts for helper-backed workflow forms.

## Scope

This design includes:

- revising `SPEC-004` structure and judgment vocabulary;
- adding proof-shaped expression and pattern semantics;
- clarifying failure ownership and helper contracts;
- adding determinism and invariant sections;
- adding planning/task artifacts and changelog bookkeeping.

This design does **not** include:

- Rust implementation changes;
- Lean proof implementation;
- a small-step semantics document;
- changing the canonical core AST shape beyond alignment edits needed in `SPEC-001`.

## Expected File Impact

### Must change

- [docs/spec/SPEC-004-SEMANTICS.md](../spec/SPEC-004-SEMANTICS.md)
- [CHANGELOG.md](../../CHANGELOG.md)

### Likely alignment changes

- [docs/spec/SPEC-001-IR.md](../spec/SPEC-001-IR.md)
- [docs/reference/formalization-boundary.md](../reference/formalization-boundary.md)

### Planning artifacts

- this design document;
- a task file under [docs/plan/tasks](../plan/tasks);
- a detailed implementation plan under [docs/plans](.).

## Risks and Mitigations

### Risk: over-scoping into provider implementation detail

Mitigation: specify helper contracts and laws, not operational algorithms for every provider.

### Risk: cross-spec drift with `SPEC-001`

Mitigation: perform a dedicated alignment pass after the main `SPEC-004` rewrite, focusing on expressions, patterns, and canonical names.

### Risk: apparent proof-readiness without usable theorem boundaries

Mitigation: add an explicit determinism/nondeterminism section and enumerate intended theorem targets in `SPEC-004` itself.

### Risk: editorial sprawl

Mitigation: move repeated explanatory prose into dedicated conventions and meta-property sections rather than attaching it to individual rules.

## Recommended Execution Phases

1. Create the task-backed planning artifacts.
2. Normalize the semantic backbone and judgment inventory.
3. Complete expression semantics.
4. Complete pattern semantics.
5. Formalize propagation and failure ownership.
6. Extract helper contracts.
7. Add determinism, invariants, and conformance notes.
8. Run a cross-spec consistency pass and update changelog/bookkeeping.

## Success Criteria

The design is satisfied when:

- every semantic judgment used in `SPEC-004` is declared explicitly;
- no semantically significant behavior is hidden behind undefined `eval(...)` / `bind(...)` shorthand;
- the pure fragment is proof-shaped and deterministic by construction;
- runtime-bearing nondeterminism is classified and constrained by explicit helper contracts;
- a Lean formalizer can identify the closed proof core without guessing about runtime-owned behavior.
