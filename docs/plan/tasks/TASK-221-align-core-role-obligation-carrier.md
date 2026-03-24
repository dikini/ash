# TASK-221: Align Core Role Obligation Carrier

## Status: ✅ Complete

## Description

Replace the current placeholder role-obligation lowering target with a core role-level carrier that
preserves named obligation references without fabricating workflow-obligation semantics.

## Specification Reference

- SPEC-001: Intermediate Representation (IR)
- SPEC-002: Surface Language

## Requirements

### Functional Requirements

1. Core role metadata must preserve named role-obligation references losslessly
2. The canonical IR spec must describe the new role-obligation carrier precisely
3. The role carrier must not reuse workflow-obligation semantics when only a named reference exists
4. Existing core tests must continue to pass after the role metadata update

## Files

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-core/src/lib.rs` (if exports change)
- Test: `ash-core` role metadata tests
- Modify: `CHANGELOG.md`

## TDD Steps

1. ✅ Add failing tests for lossless role-obligation metadata
2. ✅ Verify RED with focused `ash-core` tests
3. ✅ Implement the dedicated role-obligation carrier and spec updates
4. ✅ Verify GREEN with focused and broader `ash-core` / `ash-parser` checks
5. ☐ Commit

## Completion Checklist

- [x] core role metadata preserves named role-obligation references losslessly
- [x] canonical IR and surface specs describe the role-obligation carrier precisely
- [x] named role obligations no longer reuse workflow `Obligation` semantics
- [x] downstream parser lowering compiles against the new carrier without placeholder obligation fabrication
- [x] focused `ash-core` verification passed
- [x] focused downstream `ash-parser` verification passed
- [x] `CHANGELOG.md` updated

## Notes

- Observable inline-module role-definition lowering integration is deferred to [TASK-222](TASK-222-integrate-role-definition-lowering-path.md). TASK-221 only establishes the honest core/spec carrier and removes placeholder semantics from the existing lowering helper.
- Closeout status for this task is revalidated in [2026-03-23-role-convergence-closeout-audit.md](../../audit/2026-03-23-role-convergence-closeout-audit.md).

## Non-goals

- No role hierarchy support
- No runtime approval redesign
- No parser changes in this task
