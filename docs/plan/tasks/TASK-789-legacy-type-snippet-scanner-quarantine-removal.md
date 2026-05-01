# TASK-789: Legacy Type-Snippet Scanner Quarantine/Removal

## Status: ✅ Complete

## References

- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

## Dependencies

TASK-785, TASK-786, and TASK-787 proving the normal summary path works.

## Objective

Remove or fence legacy source-snippet ordinary type collection after the normal summary path is proven.

## Requirements

1. Audit all calls to source-snippet type collection, including `collect_public_type_defs_from_source`, `collect_type_identity_defs_from_source`, `extract_semicolon_snippets`, `check_module_file`, `collect_module_exports`, and runtime stdlib type discovery.
2. Remove normal-path dependence on `collect_public_type_defs_from_source` and related helpers, or introduce a temporary assertion/instrumentation gate proving they are not used for ordinary type metadata in normal checks.
3. If fallback remains, place it behind a clearly named compatibility/test-only path.
4. Add regression tests proving normal ModuleFile parsing is authoritative.
5. Add diagnostics or assertions for unexpected fallback use.

## Verification

- [x] Normal `ash check` and module loading do not depend on snippet type extraction.
- [x] Fallback scope, if any, is documented and tested.
- [x] Malformed normal type declarations report parser/semantic diagnostics rather than silent snippet skips.

## Completion Notes

- Audited source-snippet ordinary type collection call sites. `Engine::check_module_file`, `collect_module_exports`, and runtime stdlib type discovery all route ordinary type metadata through ModuleFile parsing and core semantic summaries.
- Renamed the remaining legacy ordinary type snippet helpers to `collect_public_type_defs_from_source_compat` and `collect_type_identity_defs_from_source_compat`, documenting them as compatibility-only/test-only paths and guarding them behind explicit `with_legacy_type_snippet_compat(...)` scope.
- Left `extract_semicolon_snippets` in place for non-type snippets such as builtin functions, imports, capabilities, and `pub mod` declarations.
- Added regression coverage proving a malformed `pub type` declaration without a semicolon fails authoritative ModuleFile parsing instead of being silently skipped by semicolon snippet extraction.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
