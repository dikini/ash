# TASK-1992: Verus Pilot 1 — Core Row Algebra

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1991 and the canonical row-rule owner from TASK-1986

## Description

Verify the PLAN-202 closed-row theorem set for `normalize_core_row` and
`core_row_included_in`, then measure proof and maintenance cost.

## Requirements

- Write property tests before proof/implementation adaptations.
- Define and check an explicit `CoreRow`/`CoreRowItem` spec view.
- Prove membership preservation, duplicate elimination, idempotence, normalization invariance,
  stable first-occurrence order, non-increasing length, membership/inclusion-truth permutation
  invariance, and closed inclusion reflexivity/transitivity.
- Keep open tails, ordered output equivalence, and diagnostic-payload equivalence outside this
  closed-row pilot.
- Keep ambiguous group rejection and row-non-authority visible.
- Perform one representation-preserving refactor and record proof maintenance evidence.

## TDD Steps

1. Add failing/strengthened property and mutation fixtures for the theorem set.
2. Define the Verus spec model and checked executable view.
3. Add proofs incrementally and run the pinned verifier.
4. Run focused Cargo tests, formatting, clippy, docs, and TCB gates.

## Completion Checklist

- [x] All listed closed-model theorems verify in `verification/verus/row_algebra.rs`: the pinned
  runner reports `15 verified`, `0 errors` under `--no-cheating --rlimit 120`.
- [x] No broad unreported assumption establishes correspondence: the manifest, runner scan, and
  report enumerate all logical-escape categories as empty and explicitly retain the direct
  Rust-`CoreRow` refinement as a deferred trace gap.
- [x] Existing focused property evidence remains green: `cargo test -p ash-core --test
  task_1642_core_row_normalization` reports 18 passing tests under Rust 1.96.0.
- [x] Maintenance evidence is recorded in `verification/verus/row-algebra-report.json`: the
  representation-preserving executable-test refactor, 18 focused tests, 15 verified items, and an
  explicit “LLM repair not measured” result prevent an invented repair-cost claim.

## Evidence

- `verification/verus/row-algebra-manifest.json`, `run-row-algebra.sh`, and
  `row-algebra-report.json` are separate from the TASK-1991 two-fixture spike. They reuse only
  its pinned release, shared Rust 1.96.0 baseline, and external-cache isolation.
- `verification/verus/ROW-ALGEBRA-README.md` states the exact `Seq<int>` closed-row model and its
  exclusions. It makes no direct claim about `normalize_core_row` or `core_row_included_in` until a
  Rust-to-model adapter/refinement is verified.
- The semantic graph marks the model proof verified while retaining
  `REQ-CORE-ROW-DIRECT-BRIDGE-001` as the explicit deferred direct-correspondence boundary.
- `cargo fmt --check` is clean. The focused Rust-1.96 clippy command reaches an unrelated existing
  `crates/ash-core/src/runtime_kernel.rs:2148` `clippy::unnecessary_sort_by` denial before this
  task's test target can be assessed; no production source is changed to hide that shared baseline
  issue. The actual focused 18-test property run, trace validator, docs tests/gate, TASK-1991 gate,
  and pinned TASK-1992 runner all pass.
