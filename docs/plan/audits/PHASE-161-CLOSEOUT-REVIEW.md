# Phase 161 Closeout Review

**Date:** 2026-06-20
**Scope:** PLAN-161 Core Ash IR Foundation closeout review for TASK-1631.
**Result:** PASS

## Review Focus

- Core/CPS boundary correctness.
- `.core` fixture format not drifting into surface syntax.
- Validation/lowering responsibility split.
- Stale overclaims in docs.

## Findings

No blocking or important findings.

## Evidence Reviewed

- `docs/plan/PLAN-161-CORE-ASH-IR-FOUNDATION.md`
- `docs/plan/tasks/TASK-1620-*.md` through `TASK-1630-*.md`
- `docs/spec/SPEC-099-CORE-LANGUAGE.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/reference/core-ash-text-format.md`
- `docs/reference/core-ash-lowering.md`
- `crates/ash-core/src/core_ash.rs`
- `crates/ash-core/src/core_ash_text.rs`
- `crates/ash-core/src/core_ash_validate.rs`
- `crates/ash-core/src/core_ash_lower.rs`
- `crates/ash-core/tests/task_1620_core_ash_ast.rs` through `task_1630_core_docs_consistency.rs`

## Review Notes

- Core Ash remains a distinct direct-style IR layer and lowers into existing CPS carriers; docs do not claim replacement of the CPS interpreter.
- `.core` is consistently documented and tested as a fixture/debug format, not surface Ash.
- Parser, validator, and lowering responsibilities are separated: raw Core is parsed first, validated through `ValidCoreProgram`, then lowered.
- SPEC-099 out-of-scope features remain bounded in implementation docs: surface-to-Core lowering, typeclass solving, arbitrary user-defined algebraic effects, `MultiShotPure`, direct Core `Match`, and full type checking are not claimed as implemented.
- Contract violations remain trap metadata, not effect row items or raised operations.
- Handler rows are documented and tested as local residual rows excluding the outer continuation row.

## Verification

Focused Phase 161 tests passed:

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
cargo test -p ash-core --test task_1621_core_text_format
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1625_core_validator_basic
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo test -p ash-core --test task_1628_core_to_cps_effects
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core --test task_1630_core_docs_consistency
```

Affected crate gates passed:

```bash
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
