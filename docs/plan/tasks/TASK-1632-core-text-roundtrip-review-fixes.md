# TASK-1632: Fix Core text public AST round-trip gaps

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161 review remediation

## Description

Fix review findings where the Core text serializer emits public Core AST syntax that the paired parser cannot read back.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)

## Dependencies

- [TASK-1624](TASK-1624-core-text-serializer.md)
- [TASK-1631](TASK-1631-phase-161-closeout.md)

## Requirements

### Functional Requirements

1. Add round-trip tests for serialized open rows in function and continuation types.
2. Add round-trip tests for serialized public type forms: refinement, record, and type application.
3. Extend the Core text parser to accept the canonical serializer spellings.
4. Preserve existing fixture syntax and canonical serializer output.

### Property Requirements

- `parse_core_expr(core_expr_to_string(expr)) == expr` for the covered public AST type/row forms.
- Open rows must preserve their row tail.
- The parser must remain fail-closed for unknown type and row forms.

## TDD Steps

### Step 1: Write failing review regression tests

**Files:** `crates/ash-core/tests/task_1624_core_text_serializer.rs`

Run:

```bash
cargo test -p ash-core --test task_1624_core_text_serializer
```

Expected: fail until the parser accepts the serializer's canonical type and row forms.

### Step 2: Implement parser support

**Files:** `crates/ash-core/src/core_ash_text.rs`

Add parser support for:

- row tails emitted as `tail Name`;
- `(refine Type "predicate")`;
- `(record-type (field : Type)...)`;
- `(type-app Name (Type...))`.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

Expected: serializer review regressions and affected gates pass.

## Completion Evidence

- Added serializer round-trip regression tests for open row tails, refinement types, record types, and type applications.
- Extended the Core text parser to accept the canonical serializer spellings for `tail`, `refine`, `record-type`, and `type-app`.
- Preserved existing canonical serializer output and fixture syntax.

Verified on 2026-06-20:

```bash
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
```
