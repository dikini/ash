# TASK-576: Integrate Salsa into ash-lsp-core

**Phase:** 89
**Spec:** SPEC-043
**Related:** SPEC-038
**Estimate:** 48 hours
**Status:** 📝 Planned

## Description

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine.

> **Prerequisite spike:** Before implementation, run an 8–12 hour spike to verify `ash-typeck` and `ash-parser` types satisfy Salsa's `'static + Clone + Eq + Hash + Debug` requirements. See SPEC-043 §7 for details.

## Sub-tasks

1. **Type derivability (8h):** Add `Eq + Hash` to `TypeCheckResult`, `Substitution`, `Type`, `ConstructorError`, `TypeEnvError`, `NameError`, `ResolutionError`, `TypeError`, and `ParseError`. This is a non-trivial prerequisite that may require refactoring interned types or float fields.
2. **Database setup (4h):** Add `salsa = "0.26"` to `ash-lsp-core`, define `SourceFile`, `WorkspaceRoot`, and `WorkspaceManifest` inputs, and define tracked queries per SPEC-043 §4.
3. **VFS wiring (8h):** Integrate `SalsaVfs` with atomic `DashMap::entry` get-or-insert, `didChange` input mutation, `didClose` handling, and `didChangeWatchedFiles` manifest updates.
4. **Cycle recovery (4h):** Implement salsa cycle recovery or define the DB drop-and-recreate fallback.
5. **Migration & tests (24h):** Run side-by-side with old cache, swap public API, remove old cache, and pass correctness/invalidation/performance tests.

## Requirements

1. Add `salsa = "0.26"` dependency to `ash-lsp-core`.
2. Define `SourceFile`, `WorkspaceRoot`, and `WorkspaceManifest` salsa inputs.
3. Define tracked queries:
   - `parse_file(db, file: SourceFile) -> (ModuleFile, Vec<ParseError>)`
   - `module_graph(db, manifest: WorkspaceManifest) -> ModuleGraph`
   - `type_check_file(db, file: SourceFile, manifest: WorkspaceManifest) -> (TypeCheckResult, Vec<ConstructorError>)`
   - `symbol_index(db, file: SourceFile) -> SymbolIndex`
4. Wrap the salsa database in `parking_lot::RwLock` for thread-safe VFS integration.
5. Wire VFS updates into salsa input changes (mutate existing inputs, never recreate).
6. Handle `didClose` by reverting closed workspace files to disk content.
7. Handle salsa cycles via cycle-recovery API or DB recreation fallback.
8. Handle I/O failures gracefully.
9. Keep the public `ash-lsp-core` API stable during the migration.

> **Hard prerequisites:**
> - `ash-parser` must expose `parse_surface_file(text: &str) -> (ModuleFile, Vec<ParseError>)`.
> - `ash-typeck` must expose `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)`.
> - `ash-typeck` and `ash-parser` types must derive `Eq + Hash`.

## Testing

1. Correctness: salsa output matches old cache output for sample workspaces.
2. Invalidation: editing `A.ash` recomputes only the transitive dependency closure of `A`.
3. Performance: measurable improvement on 10-file workspace.
4. Memory: salsa RSS does not exceed 2× old cache RSS for the same workspace.

## Completion Checklist

- [ ] `Eq + Hash` added to all salsa-crossing types
- [ ] Salsa database defined
- [ ] All analysis queries are tracked functions
- [ ] VFS feeds into salsa inputs atomically
- [ ] `didClose` handling specified and implemented
- [ ] LSP handlers use salsa without API breakage
- [ ] Old cache code removed
- [ ] Correctness, invalidation, and perf tests passing
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
