# TASK-1803: NOTE-020 promoted taxonomy cleanup

## Status: ✅ Complete

## Description

Clean up NOTE-020 after the target effect/type/Core/CPS redesign by marking its computation-row taxonomy as promoted and partially realized, adding cross-references to the implemented Core/CPS carriers and target specs, and sweeping stale wording around `Ash<rho, A>` and `pure = empty row`.

## Specification Reference

- `docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md`
- `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- `docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md`
- `docs/spec/SPEC-098b-TARGET-IR.md`
- `docs/spec/SPEC-099-CORE-LANGUAGE.md`
- `crates/ash-core/src/core_ash.rs`
- `crates/ash-core/src/cps.rs`

## Dependencies

- ✅ Phase 176 closeout: target-language redesign cleanup is complete.

## Requirements

1. Update NOTE-020 status from draft to promoted/partially realized.
2. Cross-reference the target specs and implemented Core/CPS carriers.
3. Clarify that `Ash<rho, A>` is explanatory semantic notation, not a committed source or Core type constructor.
4. Clarify that `Pure = empty row` is only a migration/profile shorthand and that the more precise predicates are `is_pure`, `is_total`, and `is_value_like`.
5. Seed remaining target-conformance deltas without turning NOTE-020 into a new implementation backlog.
6. Update `docs/notes/NOTE-INDEX.md` and `CHANGELOG.md`.

## TDD / documentation steps

### Step 1: Inspect

Read NOTE-020, NOTE-INDEX, target specs, and live Core/CPS carriers.

### Step 2: Patch documentation

Update NOTE-020 status, cross-references, explanatory notation wording, pure/empty-row wording, follow-up seeds, and changelog.

### Step 3: Update indexes and changelog

Update NOTE-INDEX and CHANGELOG under `[Unreleased]`.

## Verification

```text
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 - <<'PY'
    from pathlib import Path
    note = Path('docs/notes/NOTE-020-COMPUTATION-ROW-TAXONOMY.md').read_text()
    assert 'Promoted / partially realized' in note
    assert 'explanatory semantic notation' in note
    assert 'not a concrete source syntax commitment' in note
    assert 'not a new implementation backlog by itself' in note
    idx = Path('docs/notes/NOTE-INDEX.md').read_text()
    assert 'Promoted / partially realized' in idx
    changelog = Path('CHANGELOG.md').read_text()
    assert 'TASK-1803' in changelog
    PY
checklist:
  - [x] NOTE-020 status updated.
  - [x] Cross-references added.
  - [x] Stale wording swept.
  - [x] Follow-up seed added without creating a new implementation mandate.
  - [x] NOTE-INDEX updated.
  - [x] CHANGELOG updated.
```

## Notes

This task is docs-only. It does not implement new row/typechecker/runtime behavior. Remaining target-conformance deltas should be planned as future tasks only if the project chooses to close those deltas explicitly.
