---
id: ref.agents.card.stdlib_algebra
title: Stdlib Algebra Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
slice: reference-slice-3
owner: reference-corpus
last_verified: 2026-06-11
verified_against:
  git_commit: c1f53d76
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md
    - docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md
    - docs/design/DESIGN-NOTE-INTERFACE-LAWS.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-1388-stdlib-law-proof-readiness-audit.md
    - docs/plan/tasks/TASK-1389-semigroup-monoid-law-declarations.md
    - docs/plan/tasks/TASK-1390-functor-law-declarations.md
    - docs/plan/tasks/TASK-1391-applicative-law-declarations.md
    - docs/plan/tasks/TASK-1392-monad-law-declarations.md
    - docs/plan/tasks/TASK-1393-option-result-proof-declarations.md
    - docs/plan/tasks/TASK-1394-reference-test-handoff-closeout.md
  code:
    - std/src/algebra/semigroup.ash
    - std/src/algebra/monoid.ash
    - std/src/algebra/functor.ash
    - std/src/algebra/applicative.ash
    - std/src/algebra/monad.ash
    - std/src/algebra/comonad.ash
    - std/src/option.ash
    - std/src/result.ash
  tests:
    - crates/ash-parser/tests/task_1360_law_keyword_interface.rs
    - crates/ash-parser/tests/task_1362_proof_keyword_impl.rs
    - crates/ash-parser/tests/stdlib_parsing.rs
  examples:
    - std/src/algebra/functor.ash
    - std/src/algebra/applicative.ash
    - std/src/algebra/monad.ash
related:
  depends_on:
    - ref.stdlib.algebra
  explains:
    - ref.stdlib.index
    - ref.stdlib.result
    - ref.status.feature_matrix
    - ref.agents.common_confusions
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md
    - docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md
refresh_trigger:
  - reference/stdlib/algebra.md changes
  - std/src/algebra/*.ash changes
  - std/src/option.ash changes
  - std/src/result.ash changes
  - docs/spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md changes
  - docs/spec/SPEC-079-STANDARD-ALGEBRA-COMONAD-AND-KLEISLI-HELPERS.md changes
  - docs/design/DESIGN-NOTE-INTERFACE-LAWS.md changes
---

# Stdlib Algebra Card

canonical_page: ref.stdlib.algebra
canonical_page_path: ../../stdlib/algebra.md
dependency_order: stdlib-algebra
warning: Read the canonical page first. This card is derivative and must not redefine algebra interface or law semantics.

## Use

Retrieve the canonical page first, then use this card for search tags, stale-claim warnings, and edit preflight.

## Retrieval tags

- "algebra interfaces"
- "semigroup monoid functor applicative monad"
- "comonad kleisli"
- "law declarations"
- "proof declarations"
- "by test delegation"
- "source-visible laws"
- "Eq evidence"
- "option result instances"
- "do notation monad evidence"
- "interface evidence constraints"

## Stale-claim warnings

- Do not claim `by_definition` proofs are semantically validated. The audit (TASK-1388) confirmed they are syntactically accepted only.
- Do not claim generated law tests execute automatically. They are deferred to TASK-1029 / SPEC-077.
- Do not claim `Comonad` has stdlib carrier instances. It does not.
- Do not claim Kleisli helpers exist as concrete carrier wrappers. They are deferred.
- Do not claim Coapplicative exists. It is explicitly deferred.

## Edit preflight

Before editing this card or the canonical page:
1. Re-read `std/src/algebra/*.ash` to confirm current law/proof surfaces.
2. Run `crates/ash-parser/tests/task_1360_law_keyword_interface.rs` and `task_1362_proof_keyword_impl.rs` to verify parser coverage.
3. Run `crates/ash-parser/tests/stdlib_parsing.rs` to verify stdlib parse coverage.
4. Check `docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md` for the current proof policy.
