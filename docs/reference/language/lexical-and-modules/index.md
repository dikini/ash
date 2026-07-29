# Lexical Structure and Modules

[Language reference](../index.md) · [Status and coverage](../status.md) ·
[Source of truth](../source-of-truth.md)

## Page status

**Reviewed revision:** `423f603c` (refresh the owned AUDIT-206 rows before changing a
current-language claim).

**Implementation:** partial. This chapter documents accepted source and module-summary routes;
it does not establish a general admitted program route.
**Evidence:** tested. See [AUDIT-206](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#implementation-census)
rows LANG-001, LANG-002, LANG-003, and LANG-024.
**Parity:** below_spec. Current parser and Engine module-loader behavior, rather than older
workflow-era grammar documents, defines this chapter.

## In this chapter

- [Source files, names, comments, and literals](source-files-names-and-literals.md) — the
  `parse_surface_file` route and the lexical forms it preserves.
- [Modules, imports, and visibility](modules-imports-and-visibility.md) — `mod`, direct `use`
  parsing, and the Engine's separate leading-import routes.
- [Notation, expression macros, and operator sections](notation-and-expression-macros.md) —
  syntax-phase declarations and the required elaboration boundary.

## Scope boundary

The parser's complete-file entry point is
`crates/ash-parser/src/lib.rs::parse_surface_file`, which invokes
`crates/ash-parser/src/parse_module.rs::module_file`. It is the source route described here;
the standalone lexer is not an alternative language acceptance contract.

`workflow` declarations and workflow/tower carrier syntax are excluded. They are removed forms,
not examples or alternatives in this chapter. For the removal boundary, see the
[AUDIT-206 exclusion register](../../../plan/audits/AUDIT-206-implementation-backed-language-reference.md#exclusion-register).

## Related work

Function and ordinary expression semantics remain owned by
[TASK-2047](../../../plan/tasks/TASK-2047-language-reference-forms-functions-control-patterns.md).
This chapter identifies only the lexical and elaboration boundaries needed to read those later
forms; it does not transfer their type, lowering, or execution claims.
