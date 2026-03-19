# Spec Hardening for Mechanical Rust and Lean Implementations

## Goal

Define a spec-hardening program that makes the Ash language contract unambiguous enough that:

- Rust implementation tasks are driven by canonical language contracts rather than local choices,
- Lean formalization can model the same contracts with minimal reinterpretation,
- the lowered IR remains suitable for both an interpreter-first implementation and a later JIT,
- the language model is anchored in sound, established theory families rather than ad hoc semantics.

## Context

The repository now has:

- canonical feature-spec repairs in TASK-156 through TASK-160,
- explicit layer handoff references in TASK-161 through TASK-163,
- planned Rust convergence tasks in TASK-164 through TASK-176.

That is enough to expose the remaining problem clearly: the specs are much better aligned, but the
language definition is still not fully mechanical. Some semantics remain prose-heavy, some phase
boundaries still rely on interpretation, and some contracts still mix normative truth with
implementation-drift commentary.

That is a risk for both upcoming Rust convergence and later Lean formalization.

## Decision Summary

### 1. Introduce a spec-hardening gate before Rust convergence

No Rust-alignment task should begin until the normative semantics are tightened enough that
parser/lowering/type/runtime work is implementing one clear language definition.

### 2. Separate truth from migration

Documentation must be split mechanically:

- **specs** state the canonical contract only,
- **reference docs** state handoff boundaries only,
- **plans and tasks** describe implementation drift, rationale, and required work.

Normative specs should not carry implementation debt as if it were alternate semantics.

### 3. Tighten semantics around proof- and implementation-critical hotspots

The first hardening targets are:

- canonical core language and execution-neutral IR,
- explicit phase judgments and rejection boundaries,
- `receive` semantics,
- policy semantics,
- ADT dynamic semantics,
- observable runtime behavior,
- formalization boundary and proof targets.

### 4. Make literature anchors explicit

Each tightened area should identify its primary theory influences where appropriate, for example:

- lambda-calculus and typed operational semantics for expressions and core evaluation,
- process-calculus ideas for workflow communication and control transfer,
- algebraic data type and pattern-matching traditions from ML-family languages,
- System F style ideas where polymorphism or proof-oriented type structure is relevant.

This does not force Ash to copy any one system wholesale, but it prevents purely ad hoc semantics.

Lean formalization should treat the hardened canonical specs as the authoritative source of truth,
with the formalization-boundary note distinguishing those specs from migration-only artifacts such
as older reference interpreters, task files, and implementation drift notes.

## Why this design

Without another spec-tightening pass:

- Rust crates may continue to “discover” semantics locally,
- placeholders and fallback behavior may look acceptable because the spec boundary is still soft,
- Lean formalization would have to reverse-engineer intent from mixed prose and migration notes,
- future JIT work would risk finding that the IR contract was interpreter-shaped rather than
  execution-model-neutral.

The right order is therefore:

1. canonical feature specs,
2. explicit handoff references,
3. normative semantic hardening,
4. Rust convergence,
5. final implementation audit,
6. later Lean bisimulation/proof work on top of the hardened contract.

## Scope

This hardening pass covers:

- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-002-SURFACE.md`
- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/spec/SPEC-005-CLI.md`
- `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- `docs/spec/SPEC-011-REPL.md`
- `docs/spec/SPEC-013-STREAMS.md`
- `docs/spec/SPEC-014-BEHAVIOURS.md`
- `docs/spec/SPEC-016-OUTPUT.md`
- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- `docs/spec/SPEC-020-ADT-TYPES.md`

Expected new normative/reference artifacts:

- one explicit observable-behavior spec,
- one explicit formalization-boundary note,
- one readiness audit tying the hardening pass back to Rust and Lean implementation work.

## Phase Model

### Phase A: Core Semantics Hardening

Freeze:

- the canonical core language,
- which forms are core versus sugar,
- an execution-neutral IR contract that supports interpretation now and JIT later,
- explicit per-phase judgments and rejection boundaries.

### Phase B: Interaction Semantics Hardening

Tighten:

- `receive` mailbox and scheduler semantics,
- policy evaluation and verification semantics,
- ADT dynamic semantics.

### Phase C: Observable and Formalization Contracts

Add:

- one normative observable-behavior spec,
- one formalization-boundary note describing what Lean treats as authoritative and what proof
  targets later work should satisfy.

The older Lean reference interpreter sketch remains a historical artifact, not a second canonical
specification.

### Phase D: Readiness Audit

Audit whether the spec set is now:

- unambiguous enough for Rust convergence,
- structured enough for Lean modeling,
- explicit enough about execution neutrality for interpreter and future JIT work.

## Acceptance Criteria

This hardening program is complete only when all of the following are true:

- canonical specs no longer rely on implementation drift to explain semantics,
- phase boundaries are explicit enough that parser, lowering, typing, runtime, and observable
  behavior each have clearly owned rejection/error classes,
- the lowered IR contract is explicitly execution-model-neutral,
- `receive`, policies, and ADTs each have a proof-shaped and implementation-shaped canonical story,
- one observable-behavior spec exists as the single normative source for user-visible runtime
  behavior,
- one formalization-boundary note states which documents Lean should treat as normative.

## Non-goals

This design does not:

- start the Rust alignment work itself,
- require immediate Lean implementation,
- guarantee that every feature composition is already proven sound,
- rewrite the entire spec set into theorem-prover syntax in one pass.
