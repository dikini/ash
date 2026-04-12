# PLAN-026: Module Type Resolution Remediation

## Status: Draft (v2 -- revised after independent review)

## References

- **Design**: [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) (v2)
- **Spec**: [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) (v2)
- **Amends**: Engine type registration, module loader exports, CLI check command

## Goal

Fix four bugs preventing stdlib module files from being type-checked and imported:

1. Sibling type cross-references fail during `Engine::check()` → `TypeEnv::register_type()`.
2. `ash check` rejects non-workflow module files.
3. `pub mod` declarations are silently ignored (child modules not loaded).
4. `pub fn` parse failures are silently dropped.

## Task Breakdown

### TASK-539: Pre-declare type names in TypeEnv

**Estimate**: 2h  
**Spec**: SPEC-030 §3  
**Priority**: High (blocks TASK-541, TASK-542)  
**Layer**: `ash-typeck` + `ash-engine`

**Description**: Add `TypeEnv::declare_type_name()` and modify `Engine::check()` to pre-declare all imported type names before the full registration loop. This fixes sibling type cross-references at the actual failing layer.

**TDD Steps**:
1. Red: Write test asserting two types with forward reference (`A { x: B }`, `B { y: Int }`) register successfully when pre-declared.
2. Red: Write test asserting all 11 SPEC-029 types import and register without error.
3. Red: Write test asserting `List<Role>` and `Option<Message>` resolve correctly.
4. Red: Write test asserting self-referential type (`Tree { children: List<Tree> }`) registers.
5. Green: Add `TypeEnv::declare_type_name(name: &str)` that inserts into `ast_types` with a stub TypeDef.
6. Green: Modify `Engine::check()` (lib.rs:442) to pre-declare all imported type names before the register loop.
7. Verify: All existing engine and typeck tests pass.

**Files**:
- Modify: `crates/ash-typeck/src/type_env.rs` (add `declare_type_name`)
- Modify: `crates/ash-engine/src/lib.rs` (pre-declare loop before register loop)
- Add: `crates/ash-engine/tests/module_type_resolution_tests.rs`

---

### TASK-540: Load child modules on `pub mod`

**Estimate**: 2h  
**Spec**: SPEC-030 §4  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Extend `collect_module_exports` to process `pub mod <name>;` lines: resolve the child module path, recursively load its exports, and store them under the child module name for qualified path resolution. Do NOT merge into parent exports (baseline semantics preserved).

**TDD Steps**:
1. Red: Write test with `mod.ash` containing `pub mod types;` and `types.ash` containing `pub type Role`. Assert `use llm::types::Role` resolves.
2. Red: Write test asserting `use llm::Role` fails without explicit `pub use types::Role;`.
3. Red: Write test asserting `pub mod nonexistent;` reports error.
4. Green: Add `extract_pub_mod_declarations` to find `pub mod <name>;` lines.
5. Green: Add recursive loading, storing child exports keyed by child module name.
6. Green: Error on unresolvable module path.
7. Verify: Existing import tests pass. `use llm::types::Role` resolves.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`

---

### TASK-541: `ash check` module-file support

**Estimate**: 2h  
**Spec**: SPEC-030 §5  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Add a module-file check path in `ash check` that follows the SPEC-009 §4.1a `ModuleFile` model. Non-workflow files are validated for type/fn/use parse correctness with sibling type cross-reference resolution.

**TDD Steps**:
1. Red: Write test: `ash check` on file with only `pub type X = X { a: Int };` succeeds.
2. Red: Write test: `ash check` on file with invalid type reports specific error.
3. Green: Add module-file detection and validation path in `check.rs`.
4. Green: Use `collect_public_type_defs_from_source` with pre-declaration for validation.
5. Green: Format output per SPEC-030 §5.2.
6. Verify: `ash check std/src/llm/types.ash` succeeds.

**Files**:
- Modify: `crates/ash-cli/src/commands/check.rs`

---

### TASK-542: pub fn parse failure diagnostics

**Estimate**: 1h  
**Spec**: SPEC-030 §5.3  
**Priority**: Medium  

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
**Spec**: SPEC-030 §3.5, §4.4, §5.4  
**Priority**: Medium  
**Depends on**: TASK-539, TASK-540, TASK-541, TASK-542

**Description**: End-to-end verification that all LLM stdlib module files parse, resolve, and check correctly through the full `ash check` and module-loader paths.

**TDD Steps**:
1. Assert `ash check std/src/llm/types.ash` succeeds (11 types).
2. Assert `ash check std/src/llm/mod.ash` succeeds.
3. Assert `use llm::types::Role` from a workflow resolves.
4. Assert `use llm::Role` fails without `pub use types::Role;` in mod.ash, succeeds with it.
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
TASK-539 (TypeEnv pre-declare) ──┬──► TASK-540 (pub mod loading)
                                 ├──► TASK-541 (ash check module)
                                 └──► TASK-542 (pub fn diagnostics)
                                          │
TASK-540 + TASK-541 + TASK-542 ──► TASK-543 (e2e validation)
                                          │
                                          └──► TASK-544 (changelog)
```

TASK-540, TASK-541, and TASK-542 can run in parallel after TASK-539, but given the user's preference for sequential execution on dependent work, the recommended order is strictly sequential:

**TASK-539 → TASK-540 → TASK-541 → TASK-542 → TASK-543 → TASK-544**

## Changes from v1

| Item | v1 | v2 | Reason |
|------|----|----|--------|
| RC1 target | module_loader two-pass | TypeEnv pre-declare + Engine check | Review finding: real failure is in registration path |
| `pub mod` semantics | implicit parent export flattening | child load for qualified access only | Review finding: conflicts with SPEC-009/SPEC-012 |
| Cycle detection | mandatory in spec, out-of-scope in design | explicitly deferred | Review finding: internal contradiction |
| Check model | workflow-parse fallback | SPEC-009 ModuleFile model | Review finding: contradicts baseline |
| pub fn silent drop | not addressed | TASK-542 added | Review finding: additional live blocker |
| Task count | 5 (TASK-539 to 543) | 6 (TASK-539 to 544) | Added pub fn diagnostics task |

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Pre-declaration stub causes issues with later register_type | Stub is a minimal TypeDef; register_type replaces it |
| pub mod loading changes existing import resolution | Only activates for `pub mod` lines previously ignored |
| Module-file check path duplicates workflow check | Shared validation functions; module path is additive |
| TypeEnv API change breaks typeck callers | `declare_type_name` is new method; `register_type` unchanged |
