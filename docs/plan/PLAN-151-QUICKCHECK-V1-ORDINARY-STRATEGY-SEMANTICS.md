# PLAN-151: QuickCheck v1 Ordinary Strategy Semantics

**Status:** 📝 Planned
**Spec:** [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Design note:** [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Builds on:** [PLAN-150](PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md)
**Task range:** TASK-1497 through TASK-1506

## Goal

Replace the Phase 150 metadata/runner-bridge QuickCheck MVP with the target ordinary-Ash v1 model: pure `Strategy<A>` values, helper-first `GenContext`, ordinary `Arbitrary<A>` evidence, parser/typechecked strategy overrides, stable RNG/split/replay semantics, bounded recursive/weighted combinators, explicit shrink semantics, and aggregate empirical evidence history.

## Non-Goals

- no SmallCheck/Series implementation,
- no proof-producing synthesis,
- no automatic `Arbitrary` derivation,
- no effectful generators,
- no hidden provenance-based automatic structural shrinking,
- no narrow semantic/function identity cache in this phase,
- no global runner fallback registry for primitive/container defaults.

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Validate live syntax/callable/evidence constraints before implementation. | TASK-1497 |
| D2 | Freeze exact `test::quickcheck` module split, prelude contents, and root alias list. | TASK-1498 |
| D3 | Select and golden-test `ash-quickcheck-rng-v1`. | TASK-1499 |
| D4 | Remove/quarantine hidden default-strategy bridges in favor of in-scope `Arbitrary<A>`. | TASK-1500 |
| D5 | Finalize parser/typechecker override syntax and purity/type diagnostics. | TASK-1501 |
| D6 | Validate bounded recursive/weighted combinator semantics and invalid-config handling. | TASK-1502 |
| D7 | Validate failure-class-preserving shrink and sticky aggregate findings. | TASK-1503, TASK-1504 |
| D8 | Prove final-surface no-Cargo behavior and reconcile documentation/status. | TASK-1505, TASK-1506 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1497](tasks/TASK-1497-quickcheck-v1-live-syntax-and-seam-audit.md) | Audit live Ash syntax, callable fields, interface evidence, parser `with` seams, runner bridges, and cache identity seams before implementation. | 📝 Planned |
| [TASK-1498](tasks/TASK-1498-quickcheck-stdlib-module-split-and-prelude.md) | Split `test::quickcheck` into canonical submodules, define prelude contents, and expose alpha root aliases. | 📝 Planned |
| [TASK-1499](tasks/TASK-1499-gencontext-rng-and-strategy-value-core.md) | Implement helper-first `GenContext`, ordinary `Strategy<A>` value semantics, stable RNG/split helpers, and golden vectors. | 📝 Planned |
| [TASK-1500](tasks/TASK-1500-arbitrary-evidence-resolution-no-bridges.md) | Implement minimal `Arbitrary<A>` evidence resolution through ordinary in-scope imports and remove/quarantine hidden fallback registries. | 📝 Planned |
| [TASK-1501](tasks/TASK-1501-quickcheck-with-override-parser-typecheck.md) | Implement `by test quickcheck with { ... }` parser/typechecker support for pure `Strategy<T>` expressions. | 📝 Planned |
| [TASK-1502](tasks/TASK-1502-quickcheck-combinators-recursion-and-weights.md) | Implement function-based combinators: choice, weighted choice, map/project helpers, shrink wrappers, and bounded recursion. | 📝 Planned |
| [TASK-1503](tasks/TASK-1503-quickcheck-runner-generation-shrink-semantics.md) | Wire generation, per-parameter split paths, stop-first execution, failure-class-preserving shrink, and generator/shrinker error handling. | 📝 Planned |
| [TASK-1504](tasks/TASK-1504-quickcheck-seed-replay-and-aggregate-evidence.md) | Implement random seed default, external replay override, source-seed linting, run records, aggregate pass history, sticky errors, and active findings. | ✅ Complete |
| [TASK-1505](tasks/TASK-1505-quickcheck-v1-final-surface-fixtures-and-docs.md) | Add no-Cargo final-surface fixtures and user docs for ordinary strategies, overrides, recursion, shrinking, seeds, and evidence history. | ✅ Complete |
| [TASK-1510](tasks/TASK-1510-parser-fn-expressions-in-multi-field-struct-literals.md) | Fix parser support for `fn` expressions and closures in multi-field struct literals, unblocking ordinary Ash QuickCheck combinator patterns. | ✅ Complete |
| [TASK-1511](tasks/TASK-1511-deferred-combinators-ordinary-ash.md) | Implement deferred QuickCheck combinators (`one_of`, `recursive`, `append_shrink`, etc.) in ordinary Ash. Blocked on language features: let destructors, imported type unification, list primitives, closures. | 📝 Planned / Blocked |
| [TASK-1506](tasks/TASK-1506-quickcheck-v1-closeout-and-review.md) | Close out Phase 151 with broad verification, independent review, and status/changelog/reference reconciliation. | 📝 Planned |
## Implementation Order

1. TASK-1497 is mandatory and blocks code changes.
2. TASK-1498 establishes source layout and import surface before APIs are filled in.
3. TASK-1499 builds the strategy/context/RNG substrate.
4. TASK-1500 and TASK-1501 wire evidence and overrides.
5. TASK-1502 adds compositional authoring power.
6. TASK-1503 executes the model through the runner.
7. TASK-1504 records replay and aggregate evidence semantics.
8. TASK-1505 proves the final surface and docs.
9. TASK-1506 reconciles all status surfaces and review findings.

## Verification Strategy

Every implementation task must include focused Rust tests, no-Cargo `$ASH_UNDER_TEST test ...` fixtures when user-facing behavior changes, RNG/split golden-vector tests where relevant, negative bridge-leakage tests for retired metadata/fallback behavior, `cargo fmt --check`, scoped `cargo test` / `cargo clippy` gates, `git diff --check`, and scoped Markdown checks for docs changes.

## Closeout Criteria

- All TASK-1497 through TASK-1506 task files are complete with verification evidence.
- SPEC-087, PLAN-151, and PLAN-INDEX agree on scope/status.
- Phase 150 bridge surfaces are removed or explicitly quarantined as compatibility shims with negative leakage tests.
- Final-surface examples run through `$ASH_UNDER_TEST test ...` without `cargo run` as user-facing evidence.
- Evidence-history output distinguishes latest run outcome, aggregate pass history, counterexamples, sticky errors, and nondeterminism.
- CHANGELOG.md records the implemented phase.
