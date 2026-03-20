# TASK-213: Reconcile Module and Import Spec Scope

## Status: ✅ Complete

## Description

Resolve the remaining scope conflict between the module-system and import-system specs so the docs
no longer simultaneously describe `use` / `pub use` as both out-of-scope and active features.

## Specification Reference

- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-012-IMPORTS.md`

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-20-final-convergence-audit.md`

## Requirements

### Functional Requirements

1. Reconcile the scope story between `SPEC-009` and `SPEC-012`
2. Remove or rewrite text that still treats `use` / `pub use` as out-of-scope if those features
   are now canonical
3. Normalize directly related examples in the touched specs where they still use non-canonical type
   names such as `string` / `json`
4. Record any intentional future boundary that still remains after reconciliation

## Files

- Modify: `docs/spec/SPEC-009-MODULES.md`
- Modify: `docs/spec/SPEC-012-IMPORTS.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

List the remaining contradictions and non-canonical examples shared across the two specs.

### Step 2: Verify RED

Confirm the current text still contains conflicting scope claims or non-canonical examples.

### Step 3: Implement the reconciliation (Green)

Update the specs so the module/import story is coherent and the touched examples use canonical type
names.

### Step 4: Verify GREEN

Re-read the updated sections and confirm one continuous scope story remains.

### Step 5: Commit

```bash
git add docs/spec/SPEC-009-MODULES.md docs/spec/SPEC-012-IMPORTS.md CHANGELOG.md
git commit -m "docs: reconcile module and import scope"
```

## Completion Checklist

- [x] remaining `SPEC-009` / `SPEC-012` scope conflict removed
- [x] touched examples use canonical type names
- [x] any future boundary stated explicitly
- [x] `CHANGELOG.md` updated

## Outcome

Published:

- `docs/spec/SPEC-009-MODULES.md`
- `docs/spec/SPEC-012-IMPORTS.md`

Result:

- `SPEC-009` now defers `use` and `pub use` to `SPEC-012` instead of treating them as future
  module features
- touched examples now use canonical type names
- the remaining future boundary is explicitly limited to external crate dependencies and binary
  module compilation

## Non-goals

- No parser or module-loader code changes
- No redesign of the full module/import architecture beyond removing the documented conflict

## Dependencies

- Depends on: TASK-176
- Blocks: residual spec-only audit closure
