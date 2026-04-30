# TASK-576: Integrate Salsa into ash-lsp-core

**Phase:** 89
**Spec:** SPEC-043
**Related:** SPEC-038, SPEC-039
**Estimate:** 48 hours
**Status:** 📝 Planned (Not Implemented; Reconfirmed by TASK-767)

## Description

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine. TASK-767 reconfirmed that this has not been implemented: `ash-lsp-core` still uses the simple DashMap-backed `AnalysisCache`, has no `salsa` dependency, and lacks tracked `parse_file`, `module_graph`, `type_check_file`, or `symbol_index` queries.

> **Prerequisite spike:** Before implementation, run an 8–12 hour spike to verify `ash-typeck` and `ash-parser` types satisfy Salsa's `'static + Clone + Eq + Hash + Debug` requirements. See SPEC-043 §7 for details. **This spike is essential** — if core types cannot derive `Eq + Hash`, the entire task is blocked.

## Sub-tasks

1. **Type derivability (8h):** Add `Eq + Hash` to all types that cross salsa boundaries. This includes:
   - **Parser / AST:** `ParseError`, `ModuleFile`, and every `surface.rs` type (`Expr`, `Pattern`, `Literal`, `Definition`, `ModuleDecl`, `ImportDecl`, `TypeDef`, `WorkflowDef`, `PolicyExpr`, `EffectExpr`, `HandlerArm`, `MatchArm`, `RecordField`, `Constructor`, `Name`, `Span`)
   - **Type-checker results:** `TypeCheckResult`, `Substitution`, `Type`, `Effect`
   - **Type-checker errors:** `TypeError`, `ConstructorError`, `TypeEnvError`, `NameError`, `ResolutionError`, `ExhaustivenessError`, `ObligationCheckResult`
   - **Module graph:** `ModuleGraph`, `ModuleId`, `ModuleNode`
   This is a non-trivial prerequisite that may require refactoring interned types or float fields.
2. **Database setup (4h):** Add `salsa = "0.26"` to `ash-lsp-core`, define `SourceFile`, `WorkspaceRoot`, and `WorkspaceManifest` inputs, and define tracked queries per SPEC-043 §4.
3. **Define SymbolIndex (2h):** Add the `SymbolIndex` struct (document symbols, reference locations, cross-file usage index) as specified in SPEC-043 §4.3.
4. **VFS wiring (8h):** Integrate `SalsaVfs` with atomic `DashMap::entry` get-or-insert, `didChange` input mutation, `didClose` handling, and `didChangeWatchedFiles` manifest updates.
5. **Cycle recovery (4h):** Implement salsa cycle recovery or define the DB drop-and-recreate fallback.
6. **Migration & tests (22h):** Run side-by-side with old cache, swap public API, remove old cache, and pass correctness/invalidation/performance tests.

## Requirements

1. `crates/ash-lsp-core` exists and its public API is stable (delivered by SPEC-038 Phase 2).
2. Add `salsa = "0.26"` dependency to `ash-lsp-core`.
3. Define `SourceFile`, `WorkspaceRoot`, and `WorkspaceManifest` salsa inputs.
4. Define tracked queries:
   - `parse_file(db, file: SourceFile) -> (ModuleFile, Vec<ParseError>)`
   - `module_graph(db, manifest: WorkspaceManifest) -> ModuleGraph`
   - `type_check_file(db, file: SourceFile, manifest: WorkspaceManifest) -> (TypeCheckResult, Vec<ConstructorError>)`
   - `symbol_index(db, file: SourceFile) -> SymbolIndex`
5. Document the over-invalidation risk: because `type_check_file` is keyed on `file` and `manifest`, any manifest edit invalidates type-checking for every file. This is acceptable for MVP.
6. Wrap the salsa database in `parking_lot::RwLock` for thread-safe VFS integration.
7. Wire VFS updates into salsa input changes (mutate existing inputs, never recreate).
8. Handle `didClose` by reverting closed workspace files to disk content.
9. Handle salsa cycles via cycle-recovery API or DB recreation fallback.
10. Handle I/O failures gracefully.
11. Keep the public `ash-lsp-core` API stable during the migration.

> **Hard prerequisites:**
> - `ash-lsp-core` must be created and stabilized (SPEC-038 Phase 2).
> - `ash-parser` must expose `parse_surface_file(text: &str) -> (ModuleFile, Vec<ParseError>)` (SPEC-039).
> - `ash-typeck` must expose a module-level entry point such as `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)`. This API does **not** yet exist.
> - `ash-typeck`, `ash-parser`, and `ash-core` types must derive `Eq + Hash`.

## Testing

1. Correctness: salsa output matches old cache output for sample workspaces.
2. Invalidation: editing `A.ash` recomputes only the transitive dependency closure of `A`.
3. Performance: measurable improvement on 10-file workspace.
4. Memory: salsa RSS does not exceed 2× old cache RSS for the same workspace.

## Completion Checklist

- [ ] `Eq + Hash` added to all salsa-crossing types
- [ ] Salsa database defined
- [ ] `SymbolIndex` struct defined per SPEC-043 §4.3
- [ ] All analysis queries are tracked functions
- [ ] VFS feeds into salsa inputs atomically
- [ ] `didClose` handling specified and implemented
- [ ] Over-invalidation risk (manifest edits) documented
- [ ] LSP handlers use salsa without API breakage
- [ ] Old cache code removed
- [ ] Correctness, invalidation, and perf tests passing
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean


## TASK-767 Reconciliation Note

`PLAN-INDEX.md` previously marked Phase 89 / TASK-576 as done, but the task file and live code show that the Salsa migration remains planned. Before implementation, run the compatibility spike described above and decide whether Salsa is still the right mechanism after later Ash language changes. Do not begin this task until typecheck diagnostics/module-level query APIs and cross-file workspace requirements are clarified.
