# TASK-813: Sealed Domain Diagnostics and Non-Interference

## Status: ✅ Complete

## Description

Add diagnostics, negative tests, and non-interference coverage proving the sealed-domain substrate is explicit about failure and does not regress earlier phase behavior.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- [TASK-808](TASK-808-parser-surface-for-sealed-type-domains.md)
- [TASK-809](TASK-809-core-domain-kind-ids-and-summary-carriers.md)
- [TASK-810](TASK-810-domain-lowering-and-summary-versioning.md)
- [TASK-811](TASK-811-engine-domain-summary-export-import.md)
- [TASK-812](TASK-812-typeenv-domain-registration-and-validation.md)

## Dispatch

```
agent: hermes
reasoning: low
max_turns: 12
toolsets: [terminal, file]
```

## Objective

Prove the Phase 111 sealed-domain substrate is correct, explicit about failures, and non-interfering with existing language behavior.

## Requirements

1. Add parser/typechecker/engine-side negative coverage for duplicate constructors, unsupported field-annotation shapes, unknown field domains, malformed imported summaries, and constructor-visibility leakage.
2. Add non-interference coverage for Phase 109 ordinary type/module summary behavior and Phase 110 canonical type-expression/projection behavior.
3. Limit code changes to the minimal diagnostic text or assertion updates required by those tests.
4. Do not broaden semantics while writing tests.

## Files

- Add or update focused parser tests under `crates/ash-parser/tests/`
- Add or update focused engine tests under `crates/ash-engine/tests/`
- Add or update focused typechecker tests under `crates/ash-typeck/tests/`
- Update diagnostic text only where needed

## TDD Steps

1. Write failing tests first for the explicit Phase 111 failure boundaries.
2. Implement only the minimal fixes required to satisfy them.
3. Re-run focused parser/engine/typechecker tests.
4. Review the resulting diff for accidental scope creep.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-parser --test task_813_sealed_domain_diagnostics
  - cargo test -p ash-engine --test task_813_sealed_domain_non_interference
  - cargo test -p ash-typeck --test task_813_sealed_domain_registration_diagnostics
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Focused parser diagnostic tests pass
  - [ ] Focused engine non-interference tests pass
  - [ ] Focused typeck registration diagnostic tests pass
  - [ ] Clippy clean
  - [ ] Formatting clean
```

## Notes

This task is test-heavy and should not invent later semantics merely to make a test green.
