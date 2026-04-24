# TASK-691: Performance baseline for desugared act-block execution

## Status: ✅ Complete

## Description

Establish an honest initial performance baseline for the lowered/desugared act-block execution path before any later optimization work.

## Specification Reference

- PLAN-097 Track D

## Dependencies

- 📝 TASK-690: prerequisite task

## Requirements

### Functional Requirements

1. Measure representative desugared act-block execution paths.
2. Record baseline numbers and obvious hotspots.
3. Avoid premature optimization; this task is observational.

### Property Requirements (proptest)

```rust
// Add property-based tests where the task manipulates syntax lowering,
// typing invariants, or runtime sequencing that should hold across broad inputs.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that capture the target Phase-97 behavior before implementation.

### Step 2: Implement (Green)

Implement the minimal change set needed to satisfy the tests while preserving the additive Phase-97 architecture.

### Step 3: Integration (Green)

Wire the feature through all affected Ash layers honestly; do not introduce core-IR expansion beyond the frozen Phase-97 plan.

### Step 4: Property Tests (Verify)

Add or extend proptests for algebraic/lowering/runtime invariants where appropriate.

## Verification Steps

- [x] A reproducible baseline exists for future Phase-97 optimization discussions.
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets --all-features` passes cleanly
- [ ] `cargo fmt --check` passes

## Dependencies for Next Task

This task outputs substrate needed by its direct dependents in PLAN-097.

## Notes

- Phase 97 is additive.
- Preserve `Workflow::Act` behavior.
- Preserve coexistence with existing `Type::Fun(...)` unless this task explicitly narrows that boundary in docs/tests only.
- The baseline is now captured by the standalone `ash-bench` Criterion harness (`[workspace]` isolated in `crates/ash-bench/Cargo.toml`) and the new `phase97_act` benchmark covering permit-path `guard` forcing plus desugared bind-chain forcing depths 1/4/8/16.
- Repro command: `cargo bench --manifest-path crates/ash-bench/Cargo.toml --bench phase97_act -- --measurement-time 0.1 --sample-size 10`.
- Initial numbers recorded in this worktree: `guard_force_permit` ≈ 5.6 µs, `bind_chain_force_1` ≈ 9.8 µs, `bind_chain_force_4` ≈ 51.7 µs, `bind_chain_force_8` ≈ 107 µs, `bind_chain_force_16` ≈ 226 µs.
