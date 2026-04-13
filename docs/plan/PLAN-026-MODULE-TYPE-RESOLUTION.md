# PLAN-026: Module Type Resolution Remediation

## Status: Draft (v3 -- revised after independent review)

## References

- **Design**: [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) (v3)
- **Spec**: [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) (v3)
- **Amends**: Engine type registration, module loader exports, engine public API, CLI check command

## Goal

Fix four bugs preventing stdlib module files from being type-checked and imported:

1. Sibling type cross-references fail during `Engine::check()` → `TypeEnv::register_type()`.
2. `ash check` rejects non-workflow module files (no engine API for module-file checking).
3. `pub mod` declarations are silently ignored (child exports not available for re-export).
4. `pub fn` parse failures are silently dropped.

## Task Breakdown

### TASK-539: Pre-declare type names and upgrade in register_type

**Estimate**: 3h  
**Spec**: SPEC-030 §3  
**Priority**: High (blocks TASK-540, TASK-541, TASK-542)  
**Layer**: `ash-typeck` + `ash-engine`

**Description**: Add `TypeEnv::declare_type_name()` and modify `register_type()` to allow upgrading a placeholder. Then modify `Engine::check()` to pre-declare all imported type names before the full registration loop. This requires changes to both the typeck API and the engine's registration path.

**Required changes to `register_type`** (type_env.rs:487-489): The current duplicate-rejection guard must be modified to check whether the existing `ast_types` entry is a placeholder. Placeholders are replaced; non-placeholder duplicates still error.

**TDD Steps**:
1. Red: Write test asserting two types with forward reference (`A { x: B }`, `B { y: Int }`) register successfully when pre-declared then registered.
2. Red: Write test asserting all 11 SPEC-029 types import and register without error.
3. Red: Write test asserting `List<Role>` and `Option<Message>` resolve correctly.
4. Red: Write test asserting self-referential type (`Tree { children: List<Tree> }`) registers.
5. Red: Write test asserting non-placeholder duplicate still errors.
6. Green: Add `TypeEnv::declare_type_name(name: &str)` that inserts a placeholder `TypeDef` into `ast_types`.
7. Green: Modify `register_type()` to allow placeholder upgrade (type_env.rs:487-489).
8. Green: Modify `Engine::check()` (lib.rs:442) to pre-declare all imported type names before the register loop.
9. Verify: All existing engine and typeck tests pass.

**Files**:
- Modify: `crates/ash-typeck/src/type_env.rs` (add `declare_type_name`, modify `register_type`)
- Modify: `crates/ash-engine/src/lib.rs` (pre-declare loop before register loop)
- Add: `crates/ash-engine/tests/module_type_resolution_tests.rs`

---

### TASK-540: Load child modules on `pub mod` for re-export completeness

**Estimate**: 2h  
**Spec**: SPEC-030 §4  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Extend `collect_module_exports` to process `pub mod <name>;` lines: resolve the child module path relative to the current module's directory, recursively load its exports, and store them under the child module name. This enables `pub use` re-exports in the parent to find items from the child.

**Key context**: `use llm::types::Role` already resolves via `resolve_in_root` walking filesystem segments. `pub mod` loading fixes the `collect_module_exports` path so that `pub use types::Role;` in `mod.ash` can find `Role` from the child's exports.

**TDD Steps**:
1. Red: Write test: parent `mod.ash` has `pub mod types;` + `pub use types::Role;`, child `types.ash` has `pub type Role`. Assert `collect_module_exports(mod_path)` includes `Role` via re-export.
2. Red: Write test asserting child exports are NOT flattened into parent (must go through `pub use`).
3. Red: Write test asserting `pub mod nonexistent;` reports error.
4. Green: Add `extract_pub_mod_declarations` to find `pub mod <name>;` lines.
5. Green: Add recursive loading, storing child exports keyed by child module name.
6. Green: Error on unresolvable module path.
7. Verify: Existing import tests pass. `pub use types::Role;` in `mod.ash` resolves.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`

---

### TASK-541: Engine module-file check API + CLI integration

**Estimate**: 3h  
**Spec**: SPEC-030 §5  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Add `Engine::check_module_file(path)` public API for validating module files (non-workflow `.ash` files). Promote `collect_public_type_defs_from_source` from `pub(crate)` to `pub`. Update CLI `check.rs` to detect module files and call the new API.

**Required changes**:

1. **Engine** (`lib.rs`): Add `check_module_file(path: &Path) -> ModuleFileCheckResult` that:
   - Reads file source
   - Extracts `pub type` definitions via `collect_public_type_defs_from_source`
   - Pre-declares all type names, then registers all type defs (per SPEC-030 §3.1)
   - Returns type count, function count, warnings, errors

2. **Module loader** (`module_loader.rs`): Promote `collect_public_type_defs_from_source` from `pub(crate)` to `pub`.

3. **CLI** (`check.rs`): Detect module files (workflow parse fails) and call `engine.check_module_file(path)`.

**TDD Steps**:
1. Red: Write test: engine `check_module_file` on file with `pub type X = X { a: Int };` succeeds.
2. Red: Write test: engine `check_module_file` on file with invalid type reports specific error.
3. Red: Write integration test: `ash check` on module file succeeds with type count output.
4. Green: Add `ModuleFileCheckResult` type to engine.
5. Green: Implement `check_module_file` in engine.
6. Green: Promote `collect_public_type_defs_from_source` to `pub`.
7. Green: Update CLI `check_file()` to detect module files and use new API.
8. Verify: `ash check std/src/llm/types.ash` succeeds.

**Files**:
- Modify: `crates/ash-engine/src/lib.rs` (add `check_module_file`, `ModuleFileCheckResult`)
- Modify: `crates/ash-engine/src/module_loader.rs` (visibility promotion)
- Modify: `crates/ash-cli/src/commands/check.rs` (module-file detection and check path)

---

### TASK-542: pub fn parse failure diagnostics

**Estimate**: 1h  
**Spec**: SPEC-030 §5.3  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Change `parse_supported_pub_fn_callable` from silent `Option` return to `Result`, producing a diagnostic warning when a `pub fn` snippet fails to parse. This prevents silent export dropping and makes stdlib debugging tractable.

**TDD Steps**:
1. Red: Write test asserting a malformed `pub fn` produces a warning, not silent None.
2. Red: Write test asserting valid `pub fn` still exports correctly.
3. Green: Change return type to `Result<Option<ImportedCallableExport>, ParseWarning>`.
4. Green: Log warning on parse failure.
5. Verify: Existing pub fn import tests pass. Malformed fn produces diagnostic.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`

---

### TASK-543: LLM stdlib end-to-end validation

**Estimate**: 1h  
**Spec**: SPEC-030 §3.6, §4.6, §5.5  
**Priority**: Medium  
**Depends on**: TASK-539, TASK-540, TASK-541, TASK-542

**Description**: End-to-end verification that all LLM stdlib module files parse, resolve, and check correctly through the full `ash check` and module-loader paths.

**TDD Steps**:
1. Assert `ash check std/src/llm/types.ash` succeeds (11 types).
2. Assert `ash check std/src/llm/mod.ash` succeeds.
3. Assert `use llm::types::Role` from a workflow resolves (filesystem path, unchanged).
4. Assert `pub use types::Role;` in `mod.ash` correctly re-exports from child.
5. Assert `pub fn` exports from `prompt.ash` are not silently dropped.
6. Update `llm_stdlib_tests.rs` to use module loader API instead of string matching where possible.

**Files**:
- Modify: `crates/ash-engine/tests/llm_stdlib_tests.rs`

---

### TASK-544: Update CHANGELOG and task statuses

**Estimate**: 0.5h  
**Priority**: Low (gate)  
**Depends on**: TASK-543

**Description**: Update CHANGELOG.md, mark TASK-539 through TASK-543 complete in PLAN-INDEX.md, update task files.

**Files**:
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`

## Dependency Graph

```
TASK-539 (TypeEnv pre-declare + register upgrade) ──┬──► TASK-540 (pub mod loading)
                                                     ├──► TASK-541 (engine API + CLI)
                                                     └──► TASK-542 (pub fn diagnostics)
                                                              │
TASK-540 + TASK-541 + TASK-542 ──► TASK-543 (e2e validation)
                                                              │
                                                              └──► TASK-544 (changelog)
```

TASK-540, TASK-541, and TASK-542 can run in parallel after TASK-539, but given the user's preference for sequential execution on dependent work, the recommended order is strictly sequential:

**TASK-539 → TASK-540 → TASK-541 → TASK-542 → TASK-543 → TASK-544**

## Changes from v2

| Item | v2 | v3 | Reason |
|------|----|----|--------|
| D1 register_type change | not mentioned | explicitly required | Review finding: predeclare flow conflicts with duplicate rejection |
| D2 pub mod purpose | "qualified path resolution" | "re-export completeness" | Review finding: `resolve_in_root` already handles qualified paths via filesystem |
| D3 engine API | "check.rs change only" | "Engine::check_module_file + visibility promotion" | Review finding: no public API for module-file checking |
| TASK-539 scope | 2h, no register_type change | 3h, includes register_type modification | Blocks the predeclare flow |
| TASK-541 scope | 2h, check.rs only | 3h, engine + module_loader + CLI | Review finding: underplanned against current boundaries |
| Task filenames | mismatched content | aligned to content | Review finding: TASK-542/543 filenames wrong |

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| `register_type` placeholder upgrade breaks existing callers | Only changes behavior when placeholder is present; non-placeholder duplicates still error |
| `collect_public_type_defs_from_source` promotion leaks internal API | Function is pure (reads file, returns structs); safe to expose |
| `pub mod` loading changes existing import resolution | Only activates for `pub mod` lines previously ignored; filesystem resolution unchanged |
| Module-file check path duplicates workflow check | Uses separate engine API; shared TypeEnv pre-declaration logic |
