# TASK-576: Integrate Salsa into ash-lsp-core

**Phase:** 89
**Spec:** SPEC-043
**Related:** SPEC-038
**Estimate:** 48 hours
**Status:** 📝 Planned

## Description

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine.

> **Prerequisite spike:** Before implementation, run an 8–12 hour spike to verify `ash-typeck` types satisfy Salsa's `'static + Clone + Eq + Hash + Debug` requirements. See SPEC-043 §7 for details.

## Requirements

1. Add `salsa = "0.26"` dependency to `ash-lsp-core`.
2. Define `SourceFile` and `FilePath` salsa inputs.
3. Define tracked queries:
   - `parse_file(db, path) -> (ModuleFile, Vec<ParseError>)`
   - `module_graph(db, root) -> ModuleGraph`
   - `type_check_file(db, path) -> (TypeCheckResult, Vec<ConstructorError>)`
   - `symbol_index(db, path) -> SymbolIndex`
4. Wire VFS updates into salsa input changes (mutate existing inputs, never recreate).
5. Keep the public `ash-lsp-core` API stable during the migration.

> **Hard prerequisite:** `ash-typeck` must expose `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)` before this task can begin.

## Testing

1. Correctness: salsa output matches old cache output for sample workspaces.
2. Invalidation: editing `A.ash` recomputes only queries depending on `A`.
3. Performance: measurable improvement on 10-file workspace.

## Completion Checklist

- [ ] Salsa database defined
- [ ] All analysis queries are tracked functions
- [ ] VFS feeds into salsa inputs
- [ ] LSP handlers use salsa without API breakage
- [ ] Old cache code removed
- [ ] Correctness, invalidation, and perf tests passing
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
