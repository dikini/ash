# TASK-812: TypeEnv Domain Registration and Validation

## Status: 📝 Planned

## Description

Register and validate local and imported sealed domains plus marker constructors in `TypeEnv` using a declare-then-validate flow.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)

## Dependencies

- [TASK-810](TASK-810-domain-lowering-and-summary-versioning.md)
- [TASK-811](TASK-811-engine-domain-summary-export-import.md)

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Give `ash-typeck` one visibility-aware registration path for local and imported domains without widening Phase 111 into normalization or coverage logic.

## Requirements

1. Add `TypeEnv` registries for domain identities, visible domain aliases, marker constructors, and field metadata separate from ordinary type/constructor registries.
2. Register local and imported domains using a two-pass declare-then-validate flow so recursive and mutually recursive domain references do not depend on parse order.
3. Validate duplicate domain names, duplicate constructor names within a domain, field-domain references, and constructor/domain visibility consistency.
4. Preserve origin identity for imported aliases and opaque-domain registration without visible constructor sets.
5. Reject malformed imported summaries explicitly.
6. Preserve existing ordinary type/module summary registration and Phase 110 canonical type-expression/projection behavior.
7. Do not add normalization, constructor-disjointness solving, pattern coverage, or direct `type fn` semantics.

## Files

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify diagnostics helpers only as needed for domain-aware errors
- Add focused tests under `crates/ash-typeck/tests/`

## TDD Steps

1. Write failing tests for local recursive registration, imported exposed versus opaque domains, duplicate constructor rejection, field-reference validation, and malformed-summary rejection.
2. Implement the minimal registry and validation changes.
3. Re-run focused `ash-typeck` tests.
4. Confirm ordinary type/projection behavior remains stable.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_812_domain_registration_validation
  - cargo test -p ash-typeck
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Focused registration/validation tests pass
  - [ ] Full ash-typeck suite passes
  - [ ] Ordinary type/projection behavior stable
  - [ ] Clippy clean
  - [ ] Formatting clean
```

## Notes

Typechecker substrate task. Marker constructors must not enter ordinary constructor registries, and this task must not grow into normalization or coverage logic.
