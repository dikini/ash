# TASK-222: Integrate Role Definition Lowering Path

## Status: ✅ Complete

## Description

Make inline-module `role` definitions lower through the parser/module path exercised by crate
regression tests, using the new core role-obligation carrier, and remove the current
dead/placeholder role-lowering story.

## Specification Reference

- SPEC-001: Intermediate Representation (IR)
- SPEC-002: Surface Language
- SPEC-009: Modules

## Requirements

### Functional Requirements

1. Parsed inline-module role definitions must be lowerable through an honest test-only crate-internal parser/module helper path
2. Lowered roles must preserve named authorities and named obligations without placeholder semantics
3. The implementation must not leave placeholder lowering as the only regression-covered role-lowering path
4. Focused parser/lowering tests must cover valid and malformed inline-module role definitions

## Files

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-parser/src/lib.rs` (if a lowering API is exported)
- Test: parser/lowering unit tests for inline-module role definitions
- Test: module-level integration tests for the new lowering path
- Modify: `CHANGELOG.md`

## TDD Steps

1. ✅ Add failing tests for lowering parsed inline-module role definitions through the crate-internal path
2. ✅ Verify RED with focused `ash-parser` tests
3. ✅ Implement the minimal parser/core integration and remove placeholder lowering
4. ✅ Verify GREEN with focused and broader `ash-parser` checks
5. ☐ Commit

## Completion Checklist

- [x] parsed inline-module role definitions lower through a regression-covered test-only crate-internal parser/module helper path
- [x] lowered roles preserve named authorities and named obligations without placeholder semantics
- [x] parser/module helper coverage exercises the maintained crate-internal role-lowering entrypoint instead of leaving only placeholder lowering behind
- [x] same-module capability definitions preserve authority metadata during role lowering
- [x] unresolved authority names are rejected explicitly during lowering
- [x] inline-module parser recovery resynchronizes to both `capability` and `role` definitions after unknown braced items
- [x] focused and broader `ash-parser` verification passed
- [x] `CHANGELOG.md` updated

## Notes

- This task intentionally implements only the test-only crate-internal parser/module lowering path needed for honest inline-module role metadata. A separate non-test parser-facing lowering API remains out of scope. Repository-wide docs/example reconciliation remains deferred to [TASK-223](TASK-223-canonicalize-touched-role-docs-and-examples.md).
- Closeout status for this task is revalidated in [2026-03-23-role-convergence-closeout-audit.md](../../audit/2026-03-23-role-convergence-closeout-audit.md).

## Non-goals

- No repository-wide module lowering redesign
- No capability or policy parsing expansion beyond what the role path requires
- No example/doc updates in this task
