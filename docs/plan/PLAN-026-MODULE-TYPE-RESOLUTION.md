# PLAN-026: Module Type Resolution Remediation

## Status: Draft

## References

- **Design**: [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md)
- **Spec**: [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)
- **Amends**: SPEC-009 (Module System), SPEC-012 (Import System)

## Goal

Fix three bugs preventing stdlib module files from being parsed, type-checked, and imported:

1. `pub type` cross-references within a single file fail ("Unbound variable").
2. `ash check` rejects non-workflow module files outright.
3. `pub mod` declarations are silently ignored.

## Task Breakdown

### TASK-539: Two-pass type collection in module loader

**Estimate**: 3h  
**Spec**: SPEC-030 §3  
**Priority**: High (blocks TASK-541, TASK-542)

**Description**: Refactor `collect_public_type_defs_from_source` and `collect_module_exports` to register all type names in a first pass, then validate type expressions in a second pass.

**TDD Steps**:
1. Red: Write test asserting `pub type A = A { x: B }; pub type B = B { y: Int };` collects both types without error.
2. Red: Write test asserting `std/src/llm/types.ash` collects all 11 types without error.
3. Green: Add `collect_type_names_pass` that scans snippets for type names only.
4. Green: Modify `parse_public_type_defs` to accept an optional set of known type names and skip unbound-variable validation when the name is in the set.
5. Green: Wire two-pass flow into `collect_module_exports`.
6. Verify: All existing tests pass. New tests pass.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`
- Add: `crates/ash-engine/tests/module_type_resolution_tests.rs`

---

### TASK-540: Transitive `pub mod` loading

**Estimate**: 2h  
**Spec**: SPEC-030 §4  
**Priority**: High (blocks TASK-542)  
**Depends on**: TASK-539

**Description**: Extend `collect_module_exports` to process `pub mod <name>;` lines, resolve the submodule path, recursively load it, and merge its public exports.

**TDD Steps**:
1. Red: Write test with `mod.ash` containing `pub mod types;` and `types.ash` containing `pub type Role = System | User;`. Assert `collect_module_exports("mod.ash")` exports `Role`.
2. Red: Write test for cycle detection: two files that `pub mod` each other. Assert error.
3. Green: Add `extract_pub_mod_declarations` to find `pub mod <name>;` lines.
4. Green: Add recursive loading with visited-path cycle detection.
5. Green: Merge submodule exports, filtering to `pub` items only.
6. Verify: Existing tests pass. `use llm::Role` resolves through `llm/mod.ash`.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`

---

### TASK-541: `ash check` module-file support

**Estimate**: 2h  
**Spec**: SPEC-030 §5  
**Priority**: Medium  
**Depends on**: TASK-539

**Description**: Add a fallback path in `ash check` that detects non-workflow module files and validates their type/fn/use declarations.

**TDD Steps**:
1. Red: Write CLI test: `ash check std/src/llm/types.ash` succeeds.
2. Red: Write CLI test: file with only `pub type X = X { a: Int };` passes `ash check`.
3. Green: Add `check_module_file` function in `ash-cli/src/commands/check.rs`.
4. Green: Wire fallback: if workflow parse fails, try module-file parse.
5. Green: Format output per SPEC-030 §5.3.
6. Verify: `ash check std/src/llm/types.ash` outputs `[OK]`.

**Files**:
- Modify: `crates/ash-cli/src/commands/check.rs`
- Possibly modify: `crates/ash-engine/src/lib.rs` (add `check_module` method)

---

### TASK-542: Validate LLM stdlib end-to-end via `ash check`

**Estimate**: 1h  
**Spec**: SPEC-030 §3.4, §4.4, §5.4  
**Priority**: Medium  
**Depends on**: TASK-540, TASK-541

**Description**: Write end-to-end verification that the LLM stdlib modules parse, resolve, and check correctly through the full `ash check` path.

**TDD Steps**:
1. Assert `ash check std/src/llm/types.ash` succeeds (11 types).
2. Assert `ash check std/src/llm/mod.ash` succeeds.
3. Assert `use llm::types::Role` from a workflow file succeeds.
4. Assert `use llm::Role` (via mod.ash re-export) succeeds.
5. Update `crates/ash-engine/tests/llm_stdlib_tests.rs` to use module loader API instead of string matching where possible.

**Files**:
- Modify: `crates/ash-engine/tests/llm_stdlib_tests.rs`

---

### TASK-543: Update CHANGELOG and task statuses

**Estimate**: 0.5h  
**Priority**: Low (gate)  
**Depends on**: TASK-542

**Description**: Update CHANGELOG.md, mark TASK-539 through TASK-542 complete in PLAN-INDEX.md, update task files.

**Files**:
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-53[9-8]-*.md` (status → Complete)

## Dependency Graph

```
TASK-539 (two-pass types)
 ├─► TASK-540 (pub mod loading)
 │    └─► TASK-542 (e2e validation)
 ├─► TASK-541 (ash check module)
 │    └─► TASK-542 (e2e validation)
 └─► TASK-543 (changelog) ──► depends on TASK-542
```

## Execution Order

Sequential: TASK-539 → TASK-540 → TASK-541 → TASK-542 → TASK-543

TASK-540 and TASK-541 can run in parallel after TASK-539, but given the user's preference for sequential execution on dependent work, the recommended order is strictly sequential.

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Two-pass changes break existing import paths | Full regression test suite before merge |
| `pub mod` recursive loading hits stack overflow | Cycle detection with visited-path set |
| Module-file check false positives | Fallback only activates on parse failure |
| Type name shadowing between builtin and module types | Builtin types always take precedence per SPEC-009 §5.2 |
