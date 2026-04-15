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
pub struct WorkspaceRoot {
    #[return_ref]
    pub path: String,
}
```

### 4.2 Queries

```rust
#[salsa::tracked]
pub fn parse_file(db: &dyn AshDb, root: WorkspaceRoot, path: String)
    -> (ModuleFile, Vec<ParseError>)
{
    let source = db.source_file(root, path);
    // parse text
}

#[salsa::tracked]
pub fn module_graph(db: &dyn AshDb, root: WorkspaceRoot) -> ModuleGraph {
    // load crate roots from root path and build graph
}

#[salsa::tracked]
pub fn type_check_file(
    db: &dyn AshDb,
    root: WorkspaceRoot,
    path: String,
) -> (TypeCheckResult, Vec<ConstructorError>) {
    let module = parse_file(db, root.clone(), path.clone()).0;
    let graph = module_graph(db, root);
    // run ash-typeck via type_check_module_file(module, graph)
}

#[salsa::tracked]
pub fn symbol_index(db: &dyn AshDb, root: WorkspaceRoot, path: String) -> SymbolIndex {
    let module = parse_file(db, root, path).0;
    // build document symbols and reference index
}
```

> **Prerequisite:** `ash-typeck` must expose a `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph) -> (TypeCheckResult, Vec<ConstructorError>)` API before this query can be implemented. This should be delivered as part of SPEC-038 Phase 2.

### 4.3 Database Trait

```rust
#[salsa::db]
pub trait AshDb: salsa::Database {
    fn source_file(&self, root: WorkspaceRoot, path: String) -> SourceFile;
}

#[salsa::db]
#[derive(Default)]
pub struct AshDatabase {
    storage: salsa::Storage<Self>,
}
```

## 5. VFS Integration and Concurrency

The LSP server is multi-threaded (`tokio`), but a `salsa::Database` is typically `!Send`. Therefore the database must be wrapped in a synchronization primitive:

```rust
pub struct SalsaVfs {
    db: parking_lot::RwLock<AshDatabase>,
    file_inputs: DashMap<Url, SourceFile>,
    root_input: WorkspaceRoot,
}
```

- **Read requests** (hover, completion) acquire a read lock on the database.
- **Write requests** (`didChange`) acquire a write lock, call `source_file.set_text(...)`, then drop the lock.

When the LSP layer receives `textDocument/didChange`:

1. Look up the `SourceFile` salsa input for that URI (create it once on `didOpen`, store in `file_inputs`).
2. Acquire a write lock on `db`.
3. Call `source_file.set_text(&mut db, new_text)` to mutate the existing input.
4. Drop the lock.
5. Salsa automatically invalidates any tracked query that depended on that file.

```rust
fn on_did_change(&self, uri: Url, changes: &[TextDocumentContentChangeEvent]) {
    let new_text = apply_changes(&self.vfs.snapshot(&uri).source, changes);
    let source_file = self.file_inputs.get(&uri).expect("file not opened").clone();
    let mut db = self.db.write();
    source_file.set_text(&mut *db, new_text);
}
```

> **Important:** Creating a *new* salsa input on every `didChange` would change the input identity and invalidate all queries keyed by that path. Inputs must be created once and mutated via setters.

## 6. Error Handling

### 6.1 Salsa Cycles

If a query cycle is detected (e.g., `type_check_file(A)` depends on `type_check_file(B)` which depends on `A`), salsa will panic by default. The server must:
1. Wrap salsa calls in `std::panic::catch_unwind` (see SPEC-038 §16).
2. On cycle panic, return an LSP `InternalError` with message "cyclic module dependency detected".
3. Clear the cache entry for the affected file.

### 6.2 I/O Failures During `module_graph`

If `module_graph` encounters a missing `ash.toml` or unreadable file:
1. Return an empty graph (graceful degradation).
2. Emit a single workspace-level diagnostic: "Could not load crate graph: {reason}".
3. Log the full error at `WARN` level via `tracing`.

## 7. Salsa Compatibility Spike

Before committing to the full Salsa migration, run an 8–12 hour spike to verify that all relevant types can satisfy Salsa's trait requirements (`'static + Clone + Eq + Hash + Debug`):

- `TypeCheckResult` (contains `Substitution`, `TypeError`, `ObligationCheckResult`)
- `ModuleGraph` (contains `HashMap<ModuleId, ModuleNode>`)
- `ConstructorError`, `TypeEnvError`, `NameError`, `ResolutionError`, `TypeError`
- `Type`, `Substitution`, `Effect`
- **Parser types:** `ModuleFile`, `ParseError`, and all `surface.rs` types that cross salsa boundaries

The spike should:
1. Create a scratch crate with `salsa = "0.26"`.
2. Attempt to define `#[salsa::tracked] fn type_check_file(...) -> TypeCheckResult`.
3. Attempt to define `#[salsa::tracked] fn parse_file(...) -> (ModuleFile, Vec<ParseError>)`.
4. Record every missing `Eq` / `Hash` / `Clone` derive.
5. Report findings. Use this to revise this spec and TASK-576 before implementation begins.

## 8. Migration Strategy

The migration from simple cache to salsa should be **transparent** to `ash-lsp` handlers:

1. **Phase A:** Keep the MVP cache. Run `salsa` side-by-side in integration tests to verify parity.
2. **Phase B:** Swap the cache for salsa behind the same `ash-lsp-core` public API.
3. **Phase C:** Delete the old cache code.

> **Alternative:** If the spike in £7 shows that Salsa integration is straightforward, consider making Salsa the default cache from the start rather than building an LRU cache that is immediately thrown away.

## 9. Testing Strategy

1. **Correctness tests:** For a sample workspace, assert that salsa and the old cache produce identical diagnostics/symbols for every file.
2. **Invalidation tests:** Edit file `A.ash`; assert that `parse_file(A)` is recomputed but `parse_file(B)` is cached.
3. **Proptest invalidation:** Generate random sequences of `didChange` events across a 3-file workspace and assert that the number of recomputed queries never exceeds the number of files actually edited.
4. **Performance tests:** Measure type-check time for a 10-file workspace before and after the migration.

## 10. Relationship to Other Specs

- **Blocked by:**
  - SPEC-038 LSP MVP (must exist first; `ash-lsp-core` must be stable)
  - `ash-typeck` must expose `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)`
  - `ash-typeck` and `ash-parser` types must derive `Eq + Hash`
- **Follows:** SPEC-039, SPEC-040, SPEC-041 (all stable)
- **Enables:** Large-workspace LSP performance
