# TASK-1812: Align Core row taxonomy with target computation-row families

## Status: ✅ Complete

## Description

Align `CoreRow`/`CoreRowItem` naming, constructors, normalization, and text round-trip behavior with the target computation-row taxonomy used by Phase 177 surface rows. This task keeps Core as the semantic authority for row requirements before CPS lowering.

TASK-1807 found that Core already carries the Phase 177 row families and Core text rows already parse/format representative row items. This task should therefore focus on compatibility-visible naming, tests, normalization/public-summary behavior, and documenting retained legacy `Capability` terminology rather than replacing the Core row substrate wholesale.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100: Core Type Checking](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [NOTE-020: Computation Row Taxonomy](../../notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md)

## Dependencies

- TASK-1807 seam audit complete.
- TASK-1808 implementation decisions recorded.

## Requirements

### Functional Requirements

1. Review `crates/ash-core/src/core_ash.rs` row item variants against NOTE-020 and Phase 177 surface row item families.
2. Rename or alias legacy `Capability` terminology only if the change can be bounded without breaking existing phases; otherwise add explicit conversion helpers and status comments.
3. Preserve operation, resource, role, policy, channel, process, failure, evidence, group, and tail row families in Core.
4. Update `crates/ash-core/src/core_ash_text.rs` parsing/formatting to round-trip the supported taxonomy.
5. Update `crates/ash-core/src/core_ash_typecheck.rs` row well-formedness if needed.
6. Add property-style tests for row normalization/idempotence/order-insensitivity where existing row helpers support it.
7. Add text round-trip tests for representative row families.

### Property Requirements

- Core rows are computation requirements, not runtime authority.
- Core row normalization must not erase family identity.
- Compatibility shims must be documented and tested if legacy names remain.

## TDD Steps

### Step 1: Write failing Core row tests

Add tests under `crates/ash-core/tests/` for target row families, text round-trip, row tails, and normalization properties.

### Step 2: Verify RED

Run focused Core tests and confirm failures expose missing taxonomy or text behavior.

### Step 3: Implement Core row alignment

Patch Core carriers/text/typecheck helpers with minimal compatibility-preserving changes.

### Step 4: Verify GREEN

Run focused tests and `cargo test -p ash-core`.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core
  - git diff --check
checklist:
  - [x] Core row families cover Phase 177 target taxonomy.
  - [x] Core row text round-trips representative families.
  - [x] Normalization preserves family identity.
  - [x] Compatibility names are explicit if retained.
```

## Dependencies for Next Task

This task feeds TASK-1813 and TASK-1814.

## Completion Evidence

- Added operation-facing `CoreRowItem::operation` and `CoreRowItem::is_operation_requirement`
  helpers while retaining the legacy `Capability` storage variant with explicit compatibility docs.
- Added Core text parser aliases for target-facing `operation` and `op` row items; canonical
  Core text formatting remains `cap` to preserve existing fixtures and serialized debug text.
- Added focused TASK-1812 tests covering operation aliases, representative target row families,
  public summaries, normalization idempotence/family preservation, and canonical expression
  round-trip behavior.
- Verified with `cargo test -p ash-core --test task_1812_core_row_taxonomy_alignment`.
