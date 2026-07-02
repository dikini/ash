# TASK-1816: Remediate Phase 177 row syntax review findings

## Status: ✅ Complete

## Description

Fix the Phase 177 post-closeout review findings around target row syntax fidelity. This task keeps the Phase 177 boundary intact: it repairs parser and validation carriers for source row syntax without adding full source-to-Core row lowering, row-polymorphic inference, or provider/admission runtime wiring.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- [NOTE-025: Effect Identity via Sorts and Impls](../../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)

## Requirements

### Functional Requirements

1. Parse `{r}` as a whole-row variable, not as an operation item.
2. Preserve whole-row variables distinctly in surface row carriers and validation diagnostics.
3. Parse target open-row syntax such as `{PosixFs::read | r}` without requiring a comma before the tail.
4. Preserve row path separator spelling enough to distinguish source-path operation metadata (`fs.read`, `PosixFs.read`) from impl-qualified operation identities (`PosixFs::read`).
5. Keep impl-qualified identity validation fail-closed only for proven `::` identities; lowercase/source-path rows remain unresolved requirement metadata in this Phase 177 slice.
6. Update parser/typechecker tests so the target spec spellings, not only compatibility comma forms, are covered.
7. Update CHANGELOG and task evidence.

### Property Requirements

- Row variables are requirement metadata and do not grant authority.
- Separator preservation must not broaden runtime provider lookup or handler admission.
- Existing Phase 177 validation-only source-to-typechecker/Core boundary remains explicit.

## TDD Steps

1. Add failing parser tests for whole-row variables, open-row tail syntax without a comma, and separator preservation.
2. Add failing typechecker tests proving `PosixFs.read` is not accepted as an impl-qualified identity and remains metadata rather than being rewritten to `PosixFs::read`.
3. Implement the smallest parser/surface/typechecker changes needed to make those tests pass.
4. Run focused parser/typechecker tests, then affected crate tests and docs gates.
5. Obtain independent review and address findings.

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser --test task_1809_computation_row_parser
  - cargo test -p ash-parser --test task_1814_row_cross_boundary_parser
  - cargo test -p ash-typeck --test task_1810_impl_qualified_operation_row_identity
  - cargo test -p ash-typeck --test task_1811_row_validation_and_diagnostics
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Whole-row variables parse distinctly from operation/group items.
  - [x] Open-row tails parse with target `{item | r}` syntax.
  - [x] Operation-row separator spelling is preserved for validation.
  - [x] Independent review findings are resolved.
```

## Completion Evidence

- Added distinct parser/surface carriers for whole-row variables and row path separator spelling.
- Accepted target open-row tail syntax such as `{PosixFs::read | r}` without requiring the compatibility comma form.
- Kept impl-qualified operation identity validation gated on explicit `::` spelling, so dotted source-path metadata such as `PosixFs.read` remains unresolved row metadata in this Phase 177 slice.
- Addressed independent review feedback by parsing multi-character row variables such as `{effects}` as `WholeRow` while keeping predicate-like bare row names fail-closed through validation.
- Verification: `cargo fmt --check`; focused TASK-1809/TASK-1810/TASK-1811/TASK-1814 tests; `cargo test -p ash-parser`; `cargo test -p ash-typeck`; `cargo check --workspace`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `git diff --check`; `python3 tools/docs/validate_orientation_indexes.py --self-test`; `bash scripts/check-docs-gate.sh`.
