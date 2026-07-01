# PLAN-176: Deferred Cleanup after Target-Language Redesign

**Status:** ✅ Complete (9/9 tasks complete)
**Spec:** [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md); [SPEC-088: Closure Refinement and Effect-Safe Capture](../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md); [PLAN-157: List Migration Hardening](PLAN-157-LIST-MIGRATION-HARDENING.md); target-language specs [SPEC-095c](../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md), [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md), [SPEC-098c](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
**Depends on:** Phase 175 closeout and the target-language redesign/spec-hardening sequence through Phases 167-175.
**Task range:** TASK-1794 through TASK-1802.

## Goal

Retire the deferred cleanup candidates that were intentionally left behind before the target-language redesign: `Value::List` removal, module-level function visibility inside closures, remaining ordinary-Ash QuickCheck combinators, and stale Phase 152 status drift.

This is a cleanup phase, not a new language-design phase. Each task must prove that the newer target-language substrate is actually present before it removes an old bridge or closes an old deferral.

## Rationale

Several older phases left honest deferrals because the language substrate was not stable enough to implement the desired behavior without rework:

- Phase 157 / TASK-1570 deferred removing `Value::List` because the change was high-risk and touched hundreds of references.
- Phase 158 / TASK-1580 deferred module-level function visibility inside closures because the old tower model could not distinguish pure callable lookup from effectful action lifting cleanly.
- Phase 151 / TASK-1511 implemented part of the QuickCheck combinator surface, but recursive combinators remained blocked by missing language features.
- Phase 152 has historical status drift: PLAN-INDEX records completion, while the plan/task files still contain planned rows from an earlier snapshot.

After the target-language redesign, those candidates should be re-audited against the real parser, typechecker, lowering, runtime, and stdlib surfaces. The phase should implement only the candidates whose prerequisites are truly satisfied, and should refresh any still-deferred item with a current, testable reason.

## Scope

Phase 176 owns:

- a prerequisite/readiness audit for every cited deferral;
- removing `ash_core::Value::List` if the remaining references are compatibility shims, tests, or stale helpers rather than semantic authority;
- making module-level pure function lookup available inside closures without widening effect authority;
- completing or explicitly re-scoping the remaining ordinary-Ash QuickCheck combinators;
- reconciling Phase 151, Phase 152, Phase 157, and Phase 158 status surfaces after the cleanup decisions;
- negative leakage tests for every retired bridge.

## Non-goals

- No new list data model beyond the existing `Cons`/`Nil` representation.
- No broad effect-row implementation if effect rows are still absent in code; missing substrate must split to a future phase.
- No general recursive values unless the audit proves the current language already has the required semantics.
- No automatic `Arbitrary` derivation, SmallCheck/Series implementation, or proof-producing synthesis.
- No historical rewrite of old phases. Old tasks may be annotated as superseded/reconciled, but the cleanup work lands in Phase 176.

## Decision gates

| Gate | Question | Owner task | Default decision |
|---|---|---|---|
| D1 | Which old deferrals are now genuinely unblocked by the target-language redesign? | TASK-1795 | Audit first; split any still-blocked item instead of guessing. |
| D2 | Can `Value::List` be removed without losing literal/list runtime semantics? | TASK-1796, TASK-1797 | Remove only after classifying each reference as semantic, shim, or test fixture. |
| D3 | How should closures resolve module-level pure callables without treating them as capabilities? | TASK-1798 | Prefer typed callable identity/module environment lookup; reject effectful ambiguity. |
| D4 | Can recursive QuickCheck combinators be expressed as ordinary Ash now? | TASK-1799, TASK-1800 | Implement only with real language support; otherwise land a final-surface-compatible scoped API or keep a documented deferral. |
| D5 | Which stale status surfaces must be reconciled after implementation? | TASK-1801 | Patch PLAN/TASK/CHANGELOG/indices after code decisions, not before. |
| D6 | What broad gates and independent review prove bridges are retired safely? | TASK-1802 | Require positive behavior and negative leakage tests. |

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1794](tasks/TASK-1794-phase-176-plan-packet.md) | Create the Phase 176 deferred-cleanup planning packet | ✅ Complete |
| [TASK-1795](tasks/TASK-1795-deferred-cleanup-readiness-audit.md) | Audit deferred cleanup candidates and prerequisite substrate | ✅ Complete |
| [TASK-1796](tasks/TASK-1796-value-list-reference-classification.md) | Classify every `Value::List` reference before removal | ✅ Complete |
| [TASK-1797](tasks/TASK-1797-remove-value-list-runtime-variant.md) | Remove `Value::List` and route all list values through `Cons`/`Nil` | ✅ Complete |
| [TASK-1798](tasks/TASK-1798-module-function-visibility-in-closures.md) | Fix module-level pure function visibility inside closures | ✅ Complete |
| [TASK-1799](tasks/TASK-1799-quickcheck-recursive-combinator-design-audit.md) | Re-audit recursive QuickCheck combinator design against live language features | ✅ Complete |
| [TASK-1800](tasks/TASK-1800-quickcheck-recursive-combinators.md) | Implement or explicitly re-scope recursive QuickCheck combinators | ✅ Complete / Re-scoped |
| [TASK-1801](tasks/TASK-1801-stale-phase-status-reconciliation.md) | Reconcile stale Phase 151/152/157/158 status surfaces | ✅ Complete |
| [TASK-1802](tasks/TASK-1802-phase-176-closeout.md) | Close out Phase 176 with broad gates and independent review | ✅ Complete |

## Implementation order

1. TASK-1795 audits all deferrals and patches downstream task files if the live substrate differs from this plan.
2. TASK-1796 builds the `Value::List` reference map and decides whether removal is safe now.
3. TASK-1797 removes `Value::List` only after the reference map has an owner for every semantic use.
4. TASK-1798 fixes module-level function visibility inside closures with effect-authority negative tests.
5. TASK-1799 decides the QuickCheck recursive-combinator implementation shape using live language features.
6. TASK-1800 implements or honestly re-scopes the recursive combinators.
7. TASK-1801 reconciles stale old-phase status surfaces after implementation outcomes are known.
8. TASK-1802 runs broad gates, obtains independent review, fixes findings, and closes the phase.

## Acceptance criteria

- [x] Every cited old deferral has a current disposition table: implement now, supersede, split, or keep deferred with a fresh reason.
- [x] `Value::List` is removed from `ash_core::Value`, or the phase records a precise blocker with a failing/ignored guard test.
- [x] List literals, stdlib list functions, pattern matching, CLI JSON conversion, and tests use the same `Cons`/`Nil` value model.
- [x] Closures can call module-level pure functions through a typed/module environment path without importing capability authority or workflow effects.
- [x] QuickCheck recursive combinator behavior is re-scoped with SPEC-087 public names/config, fail-closed execution, and tests/docs that make the boundary explicit.
- [x] Phase 151, Phase 152, Phase 157, and Phase 158 status surfaces no longer contradict the implemented cleanup state.
- [x] Positive behavior tests and negative leakage tests cover every retired bridge.
- [x] CHANGELOG, PLAN-INDEX, task files, and any touched specs/reference docs agree on Phase 176 status.

## Verification baseline

```bash
cargo fmt --check
cargo test -p ash-core
cargo test -p ash-interp
cargo test -p ash-engine
cargo test -p ash-cli
cargo test -p ash-typeck
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

TASK-1797 and TASK-1802 should also run a repository-wide assertion that `Value::List` no longer appears in Rust source except in historical docs/changelog or explicit migration notes.

## Completion evidence

- TASK-1794 created and registered this Phase 176 planning packet, task files TASK-1794 through TASK-1802, PLAN-INDEX entries, and a CHANGELOG planning entry.
- TASK-1795/TASK-1796 readiness and `Value::List` reference classification are recorded in `../audit/PHASE-176-deferred-cleanup-readiness.md`.
- TASK-1797 removed the `Value::List` runtime enum variant and compatibility constructor, migrated semantic and constructor references to canonical `Cons`/`Nil` helpers, preserved external serialized `List` compatibility, and verified that no `Value::List` references remain in Rust source.

- TASK-1798 fixes closure/module-level helper visibility with shared module callable environments for local functions and hidden imported same-module runtime dependencies, plus non-leakage coverage for private helpers.

- TASK-1799 selected the SPEC-087 `recursive`/`recursive_with` public QuickCheck API and the future size-descending helper design, avoiding both self-referential values and hidden Rust fallbacks.

- TASK-1800 landed SPEC-087 public QuickCheck recursive names/config and explicitly re-scoped execution through a fail-closed private helper pending parser/type-metadata support for bounded ordinary-Ash recursion; the guard is covered by an execution regression.

- TASK-1801 reconciled historical status surfaces for Phase 151/TASK-1511, Phase 152, Phase 157/TASK-1570, and Phase 158/TASK-1580 against Phase 176 outcomes.


- TASK-1802 closed Phase 176 after broad gates and two independent review passes. The first review found runtime helper-family leakage and QuickCheck/status overclaim blockers; the second review found stale Phase 176 count/status rows. Both review rounds were remediated before final closeout.
