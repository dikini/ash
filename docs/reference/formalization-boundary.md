# Formalization Boundary and Proof Targets

## Status

TASK-183 boundary note.

## Purpose

This note freezes the boundary between canonical language contracts and migration-only artifacts
for future Lean work.

It exists so Lean formalization can treat the same semantic source of truth as Rust implementation
work without re-inferring meaning from plan text, old reference sketches, or current code shape.

## Normative Corpus

Future Lean work should treat these documents as authoritative for semantics:

- [SPEC-001: Intermediate Representation](../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md)
- [SPEC-020: Algebraic Data Types](../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

These documents define the canonical core, typing, dynamic semantics, ADT behavior, and
observable runtime contract. They are the semantic truth for both Rust and Lean work.

## Migration-Only Artifacts

The following artifacts are useful for implementation planning or historical context, but they are
not the canonical semantic source:

- surface syntax and parser-oriented references such as [SPEC-002](../spec/SPEC-002-SURFACE.md)
- CLI/REPL/output/tooling specs such as [SPEC-005](../spec/SPEC-005-CLI.md),
  [SPEC-011](../spec/SPEC-011-REPL.md), and [SPEC-016](../spec/SPEC-016-OUTPUT.md)
- planning artifacts under `docs/plan/`, including task files and hardening plans
- the runtime handoff references used during convergence
- [SPEC-021: Lean Reference Interpreter](../spec/SPEC-021-LEAN-REFERENCE.md), which is a
  historical reference sketch and not a normative contract

When Lean work needs source syntax or migration context, it should consult these artifacts as
guidance only. The semantics themselves come from the normative corpus above.

## Rust and Lean Relationship

Rust is the production implementation target.

Lean is the formalization target and proof vehicle for the same canonical contracts. It should model
the normative corpus directly, not invent alternate semantics or depend on current Rust naming.

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

## Initial Bisimulation Targets

The first bisimulation-style comparisons should cover:

1. Rust core evaluation and Lean core evaluation on closed canonical inputs.
2. Rust observable CLI/REPL/runtime outcomes and Lean-observed canonical outcomes for the same
   inputs.
3. Rust lowering and type-rejection boundaries versus Lean judgments over the same canonical forms.

The eventual JIT path is expected to preserve the same canonical IR meaning, but JIT-specific proof
work is out of scope for this note.

## Contract Hygiene

- Canonical specs define truth.
- Reference docs define handoff boundaries.
- Plans and task files define implementation work and migration notes.
- Recoverable failure is canonical only as explicit `Result` handling.
- `catch` is not part of the canonical language contract.

