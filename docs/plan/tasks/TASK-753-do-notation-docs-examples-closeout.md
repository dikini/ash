# TASK-753: Do-Notation Docs, Examples, and Closeout

## Status: ✅ Complete

## Description

Close out Phase 105 by updating specs, examples, changelog, and plan status, then running full verification and independent review. This task is docs/planning plus final verification, not a place to add new semantics.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md)
- [PLAN-101](../PLAN-101-GENERALIZED-TYPED-DO-NOTATION.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)
- [SPEC-050](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)

## Dependencies

- ✅ TASK-747 through TASK-752 complete.

## Requirements

1. Update examples to prefer `do:Act` / `do:Proc` and new-form `act` where appropriate.
2. Keep legacy examples only where explicitly documenting migration.
3. Patch SPEC-047 to state SPEC-054 owns generalized/new do grammar.
4. Patch SPEC-048/SPEC-050 only with narrow cross-links if implementation exposed new observable consequences.
5. Update [CHANGELOG.md](../../../CHANGELOG.md) with Phase 105 implementation summary.
6. Update [PLAN-INDEX](../PLAN-INDEX.md) task statuses and Phase 105 status honestly.
7. Run full verification.
8. Obtain independent subagent review before completion.

## TDD / Verification Steps

### Step 1: Corpus sweep

Search docs/examples for stale syntax:

```text
act {.*ret
x = .*;
```

Classify each hit as:

- legacy example intentionally retained;
- migration documentation;
- needs update.

### Step 2: Update docs/examples

Patch only the examples owned by this phase. Do not rewrite unrelated historical notes unless they now conflict with normative specs.

### Step 3: Run verification

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```

### Step 4: Independent review

Spawn a fresh subagent to verify:

- SPEC-054 compliance;
- Phase 104 non-interference;
- parser/type/runtime tests;
- docs/index/changelog consistency;
- no stale `TASK-XXX` placeholders.

## Verification Steps

- [x] Full cargo verification passes.
- [x] SPEC/docs/examples/changelog are updated.
- [x] PLAN-INDEX Phase 105 statuses are honest.
- [x] Independent review returns VERIFIED or all findings are resolved.
- [x] Final git diff contains only Phase 105 intended changes.

## Completion Notes

- Added Phase 105 examples under `examples/07-phase105/` covering explicit `do:Act`, new-form `act { ... }` sugar, explicit `proc::from_act(...)`, and legacy Act migration.
- Reconciled SPEC-047 to point new expression-level Act grammar at SPEC-054 and to keep legacy `ActBlock` as a migration carrier.
- Reconciled SPEC-054, docs/spec README, PLAN-101, PLAN-INDEX, and CHANGELOG status surfaces for Phase 105 closeout.
- Full verification passed: `cargo fmt --check && cargo test --workspace && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo doc --workspace --no-deps`.

## Dependencies for Next Task

This task closes Phase 105 and unblocks future Phase 106+ work on user-defined `Monad<M>` and higher-kinded constructor support.
