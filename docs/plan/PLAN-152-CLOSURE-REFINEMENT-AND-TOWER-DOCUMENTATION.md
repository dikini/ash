# PLAN-152: Closure Refinement and Tower Documentation

**Status:** ✅ Complete; Phase 176 reconciled stale planned rows
**Spec:** [SPEC-088: Closure Refinement and Effect-Safe Capture](../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
**Amends:** [PLAN-151](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) — Phase 151 status reconciled by Phase 176
**Builds on:** [SPEC-031](../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md), [SPEC-072](../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Task range:** TASK-1520 through TASK-1529

## Goal

Replace the blanket "no closures in pure functions" restriction with a precise capture-based effect rule, enabling natural closure patterns in pure code while preserving the strict environment lattice. Write comprehensive language reference documentation for functions, closures, and tower examples.

## Core Rule

> A closure created in context C may only capture values whose effect level ≤ C. A pure closure may not capture values produced by Act effects, capability handles, or closures with higher effect levels.

## Non-Goals

- No mutable state or mutable capture (Ash has no mutable refs)
- No cross-stratum closure serialization (deferred to process boundary spec)
- No automatic currying or partial application
- No inference of closure return types beyond current boundaries

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Audit current closure creation points and identify all capture channels | TASK-1520 |
| D2 | Design `EffectLevel` type system integration and capture analysis | TASK-1521 |
| D3 | Implement typechecker capture analysis with diagnostics | TASK-1522 |
| D4 | Update runtime to trust typechecker or enforce fallback | TASK-1523 |
| D5 | Verify all tower examples and QuickCheck combinators work | TASK-1524 |
| D6 | Write reference documentation for functions, closures, tower | TASK-1525 through TASK-1528 |
| D7 | Close out with status reconciliation and changelog | TASK-1529 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1520](tasks/TASK-1520-closure-refinement-audit-and-capture-channels.md) | Audit current closure creation points, identify capture channels, and document effect leakage scenarios | ✅ Complete |
| [TASK-1521](tasks/TASK-1521-effect-level-type-system-design.md) | Design `EffectLevel` enum, closure type extension, and capture analysis algorithm | ✅ Complete |
| [TASK-1522](tasks/TASK-1522-typechecker-capture-analysis.md) | Implement typechecker capture analysis: extract effect level from types, check captures, emit diagnostics | ✅ Complete |
| [TASK-1523](tasks/TASK-1523-runtime-capture-enforcement.md) | Update runtime to remove blanket ban, add fallback enforcement, or trust typechecker | ✅ Complete |
| [TASK-1524](tasks/TASK-1524-tower-examples-and-quickcheck-verification.md) | Verify all tower examples and deferred QuickCheck combinators work with refined closures | ✅ Complete / Phase 176 re-scoped QuickCheck recursion |
| [TASK-1525](tasks/TASK-1525-reference-functions-and-closures.md) | Write `reference/language/functions.md` with closure syntax, capture rules, and examples | ✅ Complete |
| [TASK-1526](tasks/TASK-1526-reference-tower-strata.md) | Write `reference/language/tower.md` with stratum examples, callable arrows, and boundary rules | ✅ Complete |
| [TASK-1527](tasks/TASK-1527-update-record-docs-with-closure-fields.md) | Update `reference/language/types/records.md` with closure field examples and capture rules | ✅ Complete |
| [TASK-1528](tasks/TASK-1528-cookbook-closure-patterns.md) | Write cookbook examples for closures at each stratum: pure, Act, Proc, Workflow | ✅ Complete |
| [TASK-1529](tasks/TASK-1529-phase-152-closeout.md) | Close out Phase 152 with verification, status reconciliation, and changelog | ✅ Complete |

## Implementation Order

1. TASK-1520 audits current state and identifies all capture channels
2. TASK-1521 designs the type system integration
3. TASK-1522 implements typechecker analysis (the core feature)
4. TASK-1523 updates runtime enforcement
5. TASK-1524 verifies tower examples and QuickCheck combinators
6. TASK-1525 through TASK-1528 write documentation in parallel
7. TASK-1529 closes out the phase

## Verification Strategy

Every implementation task must include:
- Focused Rust tests for capture analysis
- Negative tests for each rejection scenario
- Property tests for effect-level monotonicity
- No-Cargo fixtures for user-facing behavior
- `cargo fmt --check`, `cargo test`, `cargo clippy` gates
- `git diff --check`
- Documentation tests for all examples

## Closeout Criteria

- All TASK-1520 through TASK-1528 tasks are complete or explicitly marked with honest partial status
- SPEC-088, PLAN-152, and PLAN-INDEX agree on scope/status
- Typechecker rejects all effect-capture violations with correct diagnostics
- Runtime enforces or trusts typechecker (no double-check needed)
- Documentation covers all four strata with working examples
- CHANGELOG.md records the implemented phase
- Phase 151 remains open with its own closeout criteria

## Notes

Phase 176 reconciled the historical status drift in this file: TASK-1520 through TASK-1529 are complete in their own task files and PLAN-INDEX. Phase 151's TASK-1511 recursive-combinator remainder is now explicitly re-scoped by TASK-1799/TASK-1800 rather than left as stale closure-visibility blocker text.

The closure refinement enables:
- `fn make_adder(n) { fn(x) { n + x } }` — pure closures with pure captures
- `recursive` combinator with explicit state passing through `GenContext`
- More natural higher-order function patterns in ordinary Ash

Documentation tasks write the missing language reference for:
- Functions and closures (syntax, types, capture rules)
- Tower strata (Pure, Act, Proc, Workflow with examples)
- Record types with closure fields (update existing docs)
- Cookbook patterns (practical examples at each stratum)


## Phase 176 reconciliation note

TASK-1801 updated this historical plan so its task table matches PLAN-INDEX and the individual TASK-1520 through TASK-1529 files. QuickCheck recursive combinators are not claimed as fully behaviorally implemented here; TASK-1800 records the current fail-closed re-scope.
