# TASK-418: Tuple Variant Runtime Support and RuntimeError Reconciliation

## Status: 🟡 Ready

## Description

Implement the runtime/interpreter follow-on for tuple variants and reconcile the remaining concrete
`RuntimeError` contract drift identified after TASK-413.

By the end of this task:
- tuple constructor expressions should evaluate correctly
- tuple-variant patterns should match correctly at runtime
- observable display should reflect tuple payloads positionally
- the concrete stdlib-visible `RuntimeError` surface should be reconciled with the chosen contract,
  or explicitly preserved as a deliberate exception with matching docs and tests

## Specification Reference

- [TASK-413: Canonical Tuple Variant Syntax and ADT Contract Alignment](TASK-413-canonical-tuple-variant-syntax.md)
- [TASK-416: Tuple Variant Parser and Surface AST Substrate](TASK-416-tuple-variant-parser-and-surface-ast.md)
- [TASK-417: Tuple Variant Lowering, Typechecking, and Exhaustiveness](TASK-417-tuple-variant-lowering-and-typechecking.md)
- [SPEC-020: Algebraic Data Types](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

## Dependencies

- ✅ TASK-413 complete
- 🟡 TASK-416 ready/completes parser substrate
- 🟡 TASK-417 ready/completes lowering/typeck substrate

## Requirements

### Functional Requirements

1. Evaluate tuple-variant constructor expressions correctly in the interpreter.
2. Match tuple-variant patterns positionally in the interpreter.
3. Preserve existing unit-variant and record-variant runtime behavior.
4. Ensure observable formatting of tuple variants uses positional payload display.
5. Reconcile the remaining concrete `RuntimeError` drift across stdlib/entry/runtime docs and implementation surfaces.
6. Add tests covering:
   - tuple constructor evaluation
   - tuple-pattern runtime matching
   - nested tuple-pattern extraction
   - runtime display/observable formatting
   - `RuntimeError` contract behavior after reconciliation

### Non-Functional Requirements

1. If tuple payloads are represented internally with synthetic field names, that representation must remain internal-only.
2. Do not add positional projection syntax.
3. Keep source contract and concrete stdlib exceptions aligned; do not leave silent drift.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-core/src/value.rs`
- Modify: `crates/ash-interp/src/eval.rs`
- Modify any interpreter pattern-matching helpers used by `ash-interp`
- Modify/add tests under: `crates/ash-interp/tests/`
- Modify stdlib/runtime entry files only if `RuntimeError` reconciliation requires it
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing interpreter/runtime tests

Add tests proving tuple constructors evaluate and tuple patterns match at runtime.

### Step 2: Implement interpreter support

Add the minimum runtime support needed to preserve positional tuple-variant semantics.

### Step 3: Reconcile concrete RuntimeError contract

Align the actual stdlib/runtime entry surfaces with the chosen source contract or deliberately encode/document any exception.

### Step 4: Verify affected crate quality

Run at least:
- `cargo test -p ash-interp`
- `cargo clippy -p ash-interp --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [ ] tuple constructors evaluate correctly
- [ ] tuple patterns match correctly
- [ ] observable display updated
- [ ] `RuntimeError` contract reconciled
- [ ] tests added/updated
- [ ] `CHANGELOG.md` updated

## Notes

This task should close the implementation loop for TASK-413. If `RuntimeError` remains deliberately
record-shaped, that must be treated as an explicit documented exception rather than silent drift.
