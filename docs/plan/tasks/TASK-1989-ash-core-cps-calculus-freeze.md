# TASK-1989: Ash Core/CPS Calculus Freeze

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1986

## Description

Formalize the bounded Ash Core/CPS calculus that serves as the pivot between surface lowering,
runtime refinement, observable projection, and future proof work.

## Requirements

- Freeze the `λAsh-CPS₀` kernel first, then the gated `λAsh-Effect` extension defined by PLAN-202.
- Keep mathematical syntax/closures/state separate from Rust storage representations.
- State the theorem ladder and admitted fragment precisely.
- Reconcile SPEC-099/SPEC-099b, CPS references, execution-record contracts, and current target Rust.
- Keep the exclusions explicit and prevent Rust helper behavior from becoming an implicit axiom.
- Provide machine-readable rule identifiers and canonical examples.

## TDD Steps

1. Add well-formed/malformed calculus fixtures and rule-coverage checks.
2. Define pure/control rules before effect/handler/runtime boundary rules.
3. Add canonical derivation examples and expected projections.
4. Run conformance corpus and documentation gates.

## Completion Checklist

- [ ] Calculus syntax/state/judgments are complete for the admitted fragment.
- [ ] Rule ownership and exclusions are unambiguous.
- [ ] Theorem statements are precise enough for Verus/Lean work.
- [ ] Surface and runtime relations target the same calculus identities.
