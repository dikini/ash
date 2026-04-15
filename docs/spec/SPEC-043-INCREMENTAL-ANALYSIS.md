# SPEC-043: Incremental Analysis Engine for ash-lsp-core

## Status: Draft

## 1. Goal

Replace the simple per-request cache in `ash-lsp-core` with a `salsa`-based incremental query engine so that editing one file does not invalidate the analysis of unchanged files.

## 2. Scope

This spec covers:
1. Defining a `salsa` database for Ash analysis queries.
2. Mapping `parse_file`, `module_graph`, `type_check_file`, and `symbol_index` to salsa queries.
3. Integrating the salsa database into `ash-lsp-core` without breaking the LSP handler API.

## 3. Why Salsa

`ash-lsp-core` MVP uses an LRU cache keyed by `(Url, version)`. This means:
- Every `didChange` invalidates the edited file's cache entry.
- Cross-file dependencies are not tracked; changing `A.ash` invalidates nothing in `B.ash` even if `B` imports `A`.
- For large workspaces, this leads to excessive re-parsing and re-type-checking.

`salsa = "0.26"` is the proven solution used by `rust-analyzer`. It provides:
- Automatic dependency tracking between queries.
- Fine-grained invalidation when inputs change.
- Parallel query execution.

## 4. Salsa Database Design

### 4.1 Inputs

```rust
#[salsa::input]
pub struct SourceFile {
    #[return_ref]
    pub text: String,
}

#[salsa::input]
pub struct FilePath {
    #[return_ref]
    pub path: String,
}
```

### 4.2 Queries

```rust
#[salsa::tracked]
pub fn parse_file(db: &dyn AshDb, path: FilePath) -> (ModuleFile, Vec<ParseError>) {
    // parse text from the corresponding SourceFile input
}

#[salsa::tracked]
pub fn module_graph(db: &dyn AshDb, root: FilePath) -> ModuleGraph {
    // load crate roots and build graph
}

#[salsa::tracked]
pub fn type_check_file(db: &dyn AshDb, path: FilePath) -> (TypeCheckResult, Vec<ConstructorError>) {
    let module = parse_file(db, path).0;
    let graph = module_graph(db, workspace_root(path));
    // run ash-typeck via type_check_module_file(module, graph)
}

#[salsa::tracked]
pub fn symbol_index(db: &dyn AshDb, path: FilePath) -> SymbolIndex {
    let module = parse_file(db, path).0;
    // build document symbols and reference index
}
```

> **Prerequisite:** `ash-typeck` must expose a `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph) -> (TypeCheckResult, Vec<ConstructorError>)` API before this query can be implemented. This should be delivered as part of SPEC-038 Phase 2.

### 4.3 Database Trait

```rust
#[salsa::db]
pub trait AshDb: salsa::Database {
    fn source_file(&self, path: FilePath) -> SourceFile;
}

#[salsa::db]
#[derive(Default)]
pub struct AshDatabase {
    storage: salsa::Storage<Self>,
}
```

## 6. VFS Integration

When the LSP layer receives `textDocument/didChange`:

1. Look up the `FilePath` salsa input for that URI (create it once on `didOpen`, store in a `HashMap<Url, FilePath>`).
2. Call `source_file.set_text(&mut self.db, new_text)` to mutate the existing input.
3. Salsa automatically invalidates any tracked query that depended on that file.
4. The next LSP request (hover, diagnostics, etc.) triggers recomputation only for invalidated queries.

```rust
fn on_did_change(&mut self, uri: Url, changes: &[TextDocumentContentChangeEvent]) {
    let path = self.path_for_uri(&uri);
    let new_text = apply_changes(&self.vfs.snapshot(&uri).source, changes);
    let source_file = self.db.source_file(path);
    source_file.set_text(&mut self.db, new_text);
}
```

> **Important:** Creating a *new* `FilePath` input on every `didChange` would change the input identity and invalidate all queries keyed by that path. Inputs must be created once and mutated via setters.

## 7. Salsa Compatibility Spike

Before committing to the full Salsa migration, run an 8–12 hour spike to verify that `ash-typeck` types can satisfy Salsa's trait requirements (`'static + Clone + Eq + Hash + Debug`):

- `TypeCheckResult` (contains `Substitution`, `TypeError`, `ObligationCheckResult`)
- `ModuleGraph` (contains `HashMap<ModuleId, ModuleNode>`)
- `ConstructorError`, `TypeEnvError`, `NameError`, `ResolutionError`
- `Type`, `Substitution`, `Effect`

The spike should attempt to define a single `#[salsa::tracked] fn type_check_file(...) -> TypeCheckResult` in a scratch crate and record every missing derive. Use the findings to revise this spec and TASK-576 before implementation begins.

## 8. Migration Strategy

The migration from simple cache to salsa should be **transparent** to `ash-lsp` handlers:

1. **Phase A:** Keep the MVP cache. Run `salsa` side-by-side in integration tests to verify parity.
2. **Phase B:** Swap the cache for salsa behind the same `ash-lsp-core` public API.
3. **Phase C:** Delete the old cache code.

## 9. Testing Strategy

1. **Correctness tests:** For a sample workspace, assert that salsa and the old cache produce identical diagnostics/symbols for every file.
2. **Invalidation tests:** Edit file `A.ash`; assert that `parse_file(A)` is recomputed but `parse_file(B)` is cached.
3. **Performance tests:** Measure type-check time for a 10-file workspace before and after the migration.

## 10. Relationship to Other Specs

- **Blocked by:** SPEC-038 LSP MVP (must exist first; `ash-lsp-core` must be stable)
- **Follows:** SPEC-039, SPEC-040, SPEC-041 (all stable)
- **Enables:** Large-workspace LSP performance
