# TASK-1989: Ash Core/CPS Calculus Freeze

**Status:** Complete
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

- [x] Calculus syntax/state/judgments are complete for the admitted fragment.
- [x] Rule ownership and exclusions are unambiguous.
- [x] Theorem statements are precise enough for Verus/Lean work.
- [x] Surface and runtime relations target the same calculus identities.

## Completion evidence

- `docs/spec/ASH-CPS-CALCULUS.json` is the machine-readable `ash-cps-calculus/v1` artifact. It
  names mathematical syntax/state/judgments, kernel and gated-effect rule IDs, the resolved
  terminal-`Return` decision, admitted fragment, theorem-status ladder, examples, and trusted-base
  exclusions.
- `docs/spec/ASH-CPS-CALCULUS.md` is the compact human companion. It refines rather than replaces
  the `CORE-CPS-SYNTAX-001` owner in `CANONICAL-CORE.md`, explicitly marks Rust Core/CPS as
  prototype-only realization evidence, and prevents storage/helper behavior becoming an axiom.
- `CANONICAL-CORPUS.json` records the calculus detail as a non-owning active A1 node with a stable
  `SEM-CPS-CALCULUS-001` trace identity and A5-free default paths; `SPEC-INDEX.md` links the
  canonical detail after the canonical core.
- Verification on 2026-07-24:

  ```bash
  python3 tools/docs/validate_ash_cps_calculus.py \
    --artifact docs/spec/ASH-CPS-CALCULUS.json --format json
  python3 tools/docs/validate_canonical_corpus.py --root . \
    --manifest docs/spec/CANONICAL-CORPUS.json --format json \
    --check-reference-frontmatter --require-promotion-completeness
  python3 -m unittest discover -s tools/docs/tests -p 'test_*.py'
  python3 tools/docs/validate_orientation_indexes.py --self-test
  bash scripts/check-docs-gate.sh
  git diff --check
  ```
