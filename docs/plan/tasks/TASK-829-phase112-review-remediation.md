# TASK-829: Phase 112 Review Remediation

## Status: ✅ Complete

## Description

Reserve a post-closeout remediation slice for independent review findings before Phase 112 is considered ready for downstream SPEC-E work.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- ✅ [TASK-828](TASK-828-spec-d-closeout-docs-and-verification.md)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Reserve a post-closeout remediation slice for independent review findings before Phase 112 is considered ready for downstream SPEC-E work.

## Requirements

1. Run independent review of SPEC-060 implementation, tests, diagnostics, and status surfaces.
2. Fix any blocker/high findings in code or docs.
3. Re-run focused and broad verification affected by review findings.
4. Update TASK-829 with exact review findings and remediation evidence.
5. Only mark Phase 112 complete when review findings are closed.

## Files

- Modify files identified by independent review findings
- Modify: `docs/plan/tasks/TASK-829-phase112-review-remediation.md` with evidence

## TDD Steps

1. Write the audit/docs first; no Rust files change in this task.
2. Verify every claim against live files.
3. Re-read for scope creep before marking complete.

## Verification

```
strictness: clean
commands:
  - cargo test --all
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - cargo doc --workspace --no-deps
checklist:
  - [x] Independent review completed
  - [x] All blocker/high findings fixed or honestly deferred
  - [x] Focused and broad gates rerun after fixes
  - [x] Phase 112 statuses reconciled
```

## Notes

Task type: Review/Hardening. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.

## Independent Review Findings

Review inputs checked live against `SPEC-060`, `PLAN-108`, `PLAN-INDEX`, `type_ir.rs`, `normalizer.rs`, `type_env.rs`, TASK-818 through TASK-827 tests, and TASK-828 closeout evidence.

1. **High — Neutral blockers over-classified structurally known mismatches.** `Normalizer::definitional_equality(...)` returned `BlockedByNeutrality` whenever any neutral/projection subterm existed in a mismatch, even when the mismatch was already decided by different computation-head identities, projection identities, nominal data heads, or kind/arity metadata. This drifted from SPEC-060's normalize-and-compare contract: neutral terms block equality only when deciding the comparison would require inversion/solving under the neutral head, not when canonical identities are already unequal.
   - Remediation: added root structural-disjointness classification before blocker collection in `crates/ash-typeck/src/normalizer.rs`.
   - Regression: added `crates/ash-typeck/tests/task_829_review_remediation.rs` proving different neutral computation heads, different projection identities, different closed data heads with neutral arguments, projection rigidity mismatches, and same-head neutral/projection comparisons with closed known-unequal argument spines report `NotEqual` rather than `BlockedByNeutrality`, while open-vs-closed argument spines remain `BlockedByNeutrality` to preserve the non-inversion boundary.

2. **No blocker — Phase 112 scope boundaries remain intact.** Review found no public `type fn` syntax, no source equation parsing/lowering, no fixture equation summary export/import, no associated-family solver, and no proof-search/inversion implementation. Existing parser rejection and non-interference tests remain in TASK-827.

3. **No blocker — TypeEnv rollout remains narrow.** `TypeEnv::unify_types` / `types_equivalent_for_equality` use the guarded canonicalizable helper with fallback to the legacy unifier for current inference metas and unsupported legacy shapes; TASK-827 keeps this pinned.

## Remediation Evidence

- Code changed: `crates/ash-typeck/src/normalizer.rs` now distinguishes structurally disjoint normal-form mismatches from neutrality-blocked comparisons and forces definitional equality through full normalization.
- Code changed: `crates/ash-core/src/type_ir.rs` now requires neutral computation normal forms to carry a blocker reason, matching SPEC-060.
- Code changed: `crates/ash-typeck/src/type_env.rs` now uses a per-alias canonical-var bridge instead of hashing canonical variable names into synthetic `TypeVar` ids.
- Tests added: `crates/ash-typeck/tests/task_829_review_remediation.rs`.
- Focused verification after remediation:
  - `cargo test -p ash-typeck --test task_824_definitional_equality --test task_825_non_inverting_unification_boundary --test task_827_normalizer_diagnostics --test task_829_review_remediation` — passed (25 tests, 0 failures).

## Status Reconciliation

TASK-829 is complete. Phase 112 is complete/remediated after focused review remediation tests and broad workspace gates were rerun successfully.


## Final Verification Evidence

Post-remediation verification in `.worktrees/phase-112`:

- `cargo test -p ash-typeck --test task_829_review_remediation`: passed, 8 tests.
- `cargo test -p ash-typeck --test task_824_definitional_equality --test task_825_non_inverting_unification_boundary --test task_826_typeenv_forcing_point_rollout --test task_829_review_remediation`: passed, 27 tests.
- `cargo clippy -p ash-typeck --test task_829_review_remediation --all-features -- -D warnings`: passed.
- `cargo test --workspace`: passed after final remediation (background wait exit 0; foreground reruns timed out at the known long CLI input test before completion, so completion evidence uses the tracked background process exit code).
- `cargo fmt --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo doc --workspace --no-deps`: passed.
- `git diff --check`: passed.

TASK-828 previously recorded broad gates; the commands above were rerun after the final TASK-829 remediation changes before phase completion.
