# TASK-818: Core Normal-Form and Domain-Constructor Carriers

## Status: ✅ Complete

## Description

Add the shared normal-form/domain-constructor carrier substrate needed by the typechecker normalizer.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- 📝 [TASK-817](TASK-817-normalizer-defeq-audit-gate.md) (planned predecessor)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Add the shared normal-form/domain-constructor carrier substrate needed by the typechecker normalizer.

## Requirements

1. Add a normal-form carrier or view in crates/ash-core/src/type_ir.rs.
2. Represent sealed-domain constructor applications with DomainConstructorId and SealedDomainId, not ordinary ConstructorId.
3. Represent neutral computation apps plus neutral and rigid projections distinctly.
4. Preserve existing CanonicalTypeExpr behavior and serde compatibility where applicable.
5. Add focused ash-core tests for equality/hash/serde and constructor-vs-computation-head separation.

## Files

- Modify: `crates/ash-core/src/type_ir.rs`
- Modify: `crates/ash-core/src/lib.rs` if exports are needed
- Test: `crates/ash-core/tests/task_818_normal_form_carriers.rs`

## TDD Steps

1. Write focused failing tests for the task-owned behavior.
2. Run the focused test and confirm it fails for the expected reason.
3. Implement the smallest compiling change that passes the focused test.
4. Re-run focused tests and nearby regression suites.
5. Run formatting and the verification commands below.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-core --test task_818_normal_form_carriers
  - cargo test -p ash-core type_ir
  - cargo fmt --check
checklist:
  - [x] Core carrier tests pass
  - [x] Existing ash-core type_ir tests pass
  - [x] No marker constructor enters ordinary constructor identity carriers
```

## Completion Notes

- Added `NormalTypeExpr` and `NormalFormBlockReason` in `ash-core::type_ir` as shared structural carriers only; no normalizer logic, fixture equations, definitional equality, or `TypeEnv` adoption landed in this task.
- `NormalTypeExpr::DomainConstructorApp` carries `DomainConstructorId` plus `SealedDomainId` and remains distinct from `NormalTypeExpr::NominalApp` / ordinary `ConstructorId` identity paths.
- Neutral computation applications and projection normal forms preserve computation/projection heads, normalized argument spines, `Kind`, `ProjectionRigidity`, and optional stuck/blocking reason metadata for later normalizer/equality diagnostics.
- Focused TDD evidence: initial `cargo test -p ash-core --test task_818_normal_form_carriers` failed with unresolved `NormalTypeExpr`; after implementation the focused suite passed (5 tests).
- Verification run: `cargo test -p ash-core --test task_818_normal_form_carriers`; `cargo test -p ash-core type_ir`; `cargo fmt --check`; `git diff --check`.

## Notes

Task type: Core/Substrate. Estimated effort: 5 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
