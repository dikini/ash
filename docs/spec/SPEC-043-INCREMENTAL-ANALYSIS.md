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

#[salsa::input]
pub struct WorkspaceManifest {
    #[return_ref]
    pub contents: String,
}
```

### 4.2 Queries

Queries accept `SourceFile` inputs directly by identity; they no longer look up files via `(root, path)` string tuples.

```rust
#[salsa::tracked]
pub fn parse_file(db: &dyn AshDb, file: SourceFile)
    -> (ModuleFile, Vec<ParseError>)
{
    // parse file.text(db)
}

#[salsa::tracked]
pub fn module_graph(db: &dyn AshDb, manifest: WorkspaceManifest) -> ModuleGraph {
    // load crate roots from manifest contents and build graph
}

#[salsa::tracked]
pub fn type_check_file(
    db: &dyn AshDb,
    file: SourceFile,
    manifest: WorkspaceManifest,
) -> (TypeCheckResult, Vec<ConstructorError>) {
    let module = parse_file(db, file).0;
    let graph = module_graph(db, manifest);
    // run ash-typeck via type_check_module_file(module, graph)
}

#[salsa::tracked]
pub fn symbol_index(db: &dyn AshDb, file: SourceFile) -> SymbolIndex {
    let module = parse_file(db, file).0;
    // build document symbols and reference index
}
```

> **Prerequisites:**
> - `ash-parser` must expose `parse_surface_file(text: &str) -> (ModuleFile, Vec<ParseError>)` before this query can be implemented. See SPEC-038.
> - `ash-typeck` must expose a `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph) -> (TypeCheckResult, Vec<ConstructorError>)` API before this query can be implemented. This should be delivered as part of SPEC-038 Phase 2.
> - `ConstructorError` is the error type produced by `ash-typeck` during environment construction (see SPEC-038 §12).

### 4.3 Database Trait

```rust
#[salsa::db]
pub trait AshDb: salsa::Database {}

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
    manifest_input: WorkspaceManifest,
}
```

- **Read requests** (hover, completion) acquire a read lock on the database.
- **Write requests** (`didChange`, `didChangeWatchedFiles`) acquire a write lock, mutate the relevant salsa input, then drop the lock.
- **`didOpen`:** Use `DashMap::entry(uri).or_insert_with(...)` to atomically create the `SourceFile` input once, avoiding a TOCTOU race between the `DashMap` check and the `RwLock` write.
- **`didClose`:** If the file belongs to the workspace, revert the `SourceFile` input to the current on-disk content (so that closed files still participate in cross-file analysis with disk state). If the file is ephemeral, remove it from `file_inputs` and stop passing it to salsa queries.
- **`didChangeWatchedFiles`:** When the workspace `ash.toml` changes on disk, acquire a write lock and call `manifest_input.set_contents(&mut db, new_contents)`.

When the LSP layer receives `textDocument/didChange`:

1. Look up the `SourceFile` salsa input for that URI (created once on `didOpen`, stored in `file_inputs`).
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

> **Memory budget:** `SalsaVfs` holds every open workspace file as a salsa input. For very large workspaces (>10k files), consider a LRU eviction policy for unopened files or shard the database per crate. This is out of scope for the initial migration.

## 6. Error Handling

### 6.1 Salsa Cycles

If a query cycle is detected (e.g., `type_check_file(A)` depends on `type_check_file(B)` which depends on `A`), the server must recover gracefully. **Preferred:** Use salsa's cycle-recovery API (`#[salsa::tracked(cycle_fn = recover_cycle, cycle_initial = init_cycle)]`) to return a recovered value rather than panicking. **Fallback:** If cycle recovery cannot be used, drop and recreate the `AshDatabase`, re-register all `SourceFile`, `WorkspaceRoot`, and `WorkspaceManifest` inputs, and return an LSP `InternalError` with message "cyclic module dependency detected".

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

> **Tracked prerequisite:** Adding `Eq + Hash` to `TypeCheckResult`, `Substitution`, `Type`, and all error types (`ConstructorError`, `TypeEnvError`, `NameError`, `ResolutionError`, `TypeError`, `ParseError`) is a non-trivial prerequisite that may require refactoring interned types or replacing `f32`/`f64` fields with ordered wrappers. Track this as a separate sub-task in TASK-576.

## 8. Migration Strategy

The migration from simple cache to salsa should be **transparent** to `ash-lsp` handlers:

1. **Phase A:** Keep the MVP cache. Run `salsa` side-by-side in integration tests to verify parity.
2. **Phase B:** Swap the cache for salsa behind the same `ash-lsp-core` public API.
3. **Phase C:** Delete the old cache code.

> **Alternative:** If the spike in §7 shows that Salsa integration is straightforward, consider making Salsa the default cache from the start rather than building an LRU cache that is immediately thrown away.

## 9. Testing Strategy

1. **Correctness tests:** For a sample workspace, assert that salsa and the old cache produce identical diagnostics/symbols for every file.
2. **Invalidation tests:** Edit file `A.ash`; assert that `parse_file(A)` is recomputed but `parse_file(B)` is cached.
3. **Proptest invalidation:** Generate random sequences of `didChange` events across a 3-file workspace and assert that the exact set of recomputed queries equals the transitive dependency closure of the edited files (i.e., no over-invalidation and no under-invalidation).
4. **Performance tests:** Measure type-check time for a 10-file workspace before and after the migration.
5. **Memory tests:** Assert that the salsa database resident set size does not exceed 2× the old cache RSS for the same workspace under steady state.

## 10. Relationship to Other Specs

- **Blocked by:**
  - SPEC-038 LSP MVP (must exist first; `ash-lsp-core` must be stable)
  - `ash-parser` must expose `parse_surface_file(text: &str) -> (ModuleFile, Vec<ParseError>)`
  - `ash-typeck` must expose `type_check_module_file(module: &ModuleFile, graph: &ModuleGraph)`
  - `ash-typeck` and `ash-parser` types must derive `Eq + Hash` (see §7 prerequisite task)
- **Follows:** SPEC-039, SPEC-040, SPEC-041 (all stable)
- **Enables:** Large-workspace LSP performance
