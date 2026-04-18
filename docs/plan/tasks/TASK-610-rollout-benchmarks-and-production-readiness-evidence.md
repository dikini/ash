# TASK-610: Rollout, Benchmarks, and Production-Readiness Evidence

## Status: ✅ Complete

## Description

Add the rollout controls and evidence required to call the integrated small-step/lifting work production-quality: runtime-selection policy, benchmarks, CI-facing verification commands, and docs/changelog/spec alignment.

## Specification Reference

- `docs/design/DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`
- `docs/design/DESIGN-028-STATEMENT-LIFTING.md`
- `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`

## Dependencies

- 📝 TASK-607: Small-Step Runtime Parity and Gap Closure
- 📝 TASK-609: Effect Classification Alignment for Lifting

## Requirements

1. Runtime rollout policy is explicit (feature flag or documented default-selection boundary).
2. Benchmarks/report artifacts exist for IR/runtime claims.
3. `CHANGELOG.md`, task files, and plan/spec references reflect reality.
4. CI-relevant verification commands for this feature set are documented and runnable.

## TDD Steps

1. Add/enable benchmark harnesses for representative workflows.
2. Add rollout/configuration controls and tests if needed.
3. Update docs/changelog/task statuses/spec references.
4. Run the full verification suite and record evidence.

## Verification Steps

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] benchmark/report command documented and reproducible

## Notes

Do not claim production quality without rollout policy and evidence.