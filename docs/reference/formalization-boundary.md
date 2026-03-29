# Formalization Boundary and Proof Targets

## Status

TASK-183 boundary note.

## Purpose

This note freezes the boundary between canonical language contracts, authoritative source/handoff
contracts, and historical artifacts for future Lean work.

It exists so Lean formalization can treat the same semantic source of truth as Rust implementation
work without re-inferring meaning from plan text, old reference sketches, or current code shape.

## Canonical Semantic Corpus

Lean should treat these documents as authoritative for language semantics:

- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-020: Algebraic Data Types](../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

These documents define the canonical core, typing, dynamic semantics, ADT behavior, and
observable runtime contract. They are the semantic truth for both Rust and Lean work.

## Authoritative Source and Handoff Contracts

Lean should also treat these documents as authoritative for their own layer-specific contracts:

- [SPEC-002: Surface Language](../spec/SPEC-002-SURFACE.md)
- [SPEC-005: CLI Specification](../spec/SPEC-005-CLI.md)
- [SPEC-011: REPL](../spec/SPEC-011-REPL.md)
- [SPEC-016: Output Capabilities](../spec/SPEC-016-OUTPUT.md)
- [Surface-to-Parser Contract](surface-to-parser-contract.md)
- [Parser-to-Core Lowering Contract](parser-to-core-lowering-contract.md)
- [Type-to-Runtime Contract](type-to-runtime-contract.md)
- [Runtime Observable Behavior Contract](runtime-observable-behavior-contract.md)

These contracts are authoritative for source syntax, lowering handoff, CLI/REPL observability,
and runtime handoff boundaries. They do not replace the canonical semantic corpus above, but Lean
work should still use them as the contract for their respective layers.

## Historical and Migration-Only Artifacts

The following artifacts are useful for implementation planning or historical context, but are not
the canonical semantic source:

- planning artifacts under `docs/plan/`, including task files and hardening plans
- the old reference interpreter sketch at [Lean Reference Interpreter](../spec/SPEC-021-LEAN-REFERENCE.md)

When Lean work needs migration context, it should consult these artifacts as guidance only. The
semantics themselves come from the canonical semantic corpus, while source and handoff contracts
remain authoritative for their respective layers.

## Rust and Lean Relationship

Rust is the production implementation target.

Lean is the formalization target and proof vehicle for the same canonical contracts. It should model
the canonical semantic corpus directly, use authoritative source/handoff contracts for syntax and
layer boundaries, and not invent alternate semantics or depend on current Rust naming.

Where Rust and Lean differ, the canonical specs win. Divergence belongs in migration tasks and
implementation work, not in the formalization boundary.

## Initial Proof Targets

The first proof targets should be small and judgment-shaped:

1. Lowering soundness: surface forms lower to canonical IR without changing meaning.
2. Typing preservation: well-typed core programs remain well-typed across evaluation steps where
   the semantics are defined to continue.
3. ADT constructor and pattern soundness: constructor-shaped values, pattern matching, and
   exhaustiveness behave consistently.
4. Observable behavior preservation: the observable outcomes frozen in SPEC-021 are stable across
   equivalent implementations.
5. Explicit recoverable failure: recoverable failures are represented and handled through `Result`
   dataflow, not exceptional `catch` control flow.

For SPEC-004 specifically, these initial proof targets now align with its explicit judgment and
meta-property structure:

- workflow outcomes and propagation conventions;
- pure expression and pattern determinism;
- helper-backed conformance obligations and permitted nondeterminism;
- semantic invariants around effect accumulation, trace preservation, and rejection ownership.

## Initial Bisimulation Targets

The first bisimulation-style comparisons should cover:

1. Rust core evaluation and Lean core evaluation on closed canonical inputs.
2. Rust observable CLI/REPL/runtime outcomes and Lean-observed canonical outcomes for the same
   inputs.
3. Rust lowering and type-rejection boundaries versus Lean judgments over the same canonical forms.

The eventual JIT path is expected to preserve the same canonical IR meaning, but JIT-specific proof
work is out of scope for this note.

## Contract Hygiene

- Canonical specs define semantic truth.
- Source/handoff contracts define layer-specific authority.
- Plans and task files define implementation work and migration notes.
- Recoverable failure is canonical only as explicit `Result` handling.
- `catch` is not part of the canonical language contract.
