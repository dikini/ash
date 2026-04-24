# Track A Tasks: Module Resolution Engine

## TASK-654: Module Resolver — Failing Test Suite

**Spec:** SPEC-009, SPEC-012, SPEC-030
**Track:** A
**Depends on:** D1 resolution (resolver stays in ash-engine)
**Est. Hours:** 4-6

Write the failing test suite that defines the expected module resolution behavior:

1. Local-tree resolution: `use sibling::Type` resolves to `./sibling.ash`
2. Nested multi-file: `use crate::foo::bar` with `foo/mod.ash` + `foo/bar.ash`
3. Stdlib resolution: `use std::json` resolves to `std/src/json.ash`
4. `ASH_LIBRARY_PATH` search order: local tree → library dirs → stdlib root
5. Version-qualified imports: `use math@1::vector` with single-version enforcement
6. Import cycle detection: circular `use` produces clear error
7. Missing module produces clear error with search paths listed

**Files:**
- Create: `crates/ash-engine/tests/module_resolution.rs`
- Create: `crates/ash-cli/tests/run_imports.rs`

**Verification:** All tests fail (red phase).

---

## TASK-655: Module Resolver — Core Implementation

**Spec:** SPEC-009, SPEC-012
**Track:** A
**Depends on:** TASK-654
**Est. Hours:** 8-12

Implement the graph-backed module resolver in `ash-engine`:

1. `ModuleResolver` struct with configuration (root path, library paths, stdlib root)
2. `resolve_module_path()` — filesystem resolution with deterministic precedence
3. `load_module_graph()` — recursive graph loading from root file
4. Cycle detection (visited set per resolution chain)
5. Version enforcement (reject conflicting version requests for same library)
6. `ASH_LIBRARY_PATH` parsing and search-order implementation

**Files:**
- Create: `crates/ash-engine/src/module_resolver.rs`
- Modify: `crates/ash-engine/src/lib.rs` (exports)
- Modify: `crates/ash-engine/src/error.rs` (resolution error variants)

**Verification:** Engine tests from TASK-654 pass.

---

## TASK-656: Module Resolver — Stdlib as Resolver Root

**Spec:** SPEC-009 §2.1
**Track:** A
**Depends on:** TASK-655
**Est. Hours:** 3-4

Wire the stdlib into the module resolver so `use result::Ok` etc. resolve:

1. Add stdlib root discovery (compile-time path, runtime `ASH_STDLIB_PATH` override)
2. Map `std/src/*.ash` to module paths (`std::json` → `std/src/json.ash`)
3. Map `std/src/*/mod.ash` to submodule paths (`std::io::fs` → `std/src/io/fs.ash`)
4. On-demand loading: only parse and register modules that are actually `use`d
5. Integrate with engine's existing `load_ordinary_file()` for each resolved module

**Files:**
- Modify: `crates/ash-engine/src/module_resolver.rs`
- Modify: `crates/ash-engine/src/lib.rs`

**Verification:** `use std::json` resolves; `use std::io::fs` resolves; unknown stdlib modules produce clear error.

---

## TASK-657: Module Resolver — Thread into Engine Execution

**Spec:** SPEC-010
**Track:** A
**Depends on:** TASK-655, TASK-656
**Est. Hours:** 6-8

Replace the current single-source execution path with graph-backed execution:

1. `Engine::run_file()` uses `ModuleResolver` to build the full module graph
2. `Engine::check()` receives imported types/callables from resolved modules
3. `Engine::execute()` has all imported definitions available in `RuntimeState`
4. Entry bootstrap path uses same resolver (with its narrow stdlib subset)
5. Clear error messages for resolution failures at each stage

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/execute.rs`
- Modify: `crates/ash-engine/src/check.rs`
- Modify: `crates/ash-engine/src/entry.rs`
- Modify: `crates/ash-engine/src/parse.rs`

**Verification:** All engine tests pass, including multi-file scenarios.

---

## TASK-658: Module Resolver — CLI Integration

**Spec:** SPEC-005
**Track:** A
**Depends on:** TASK-657
**Est. Hours:** 4-6

Wire the graph-backed loader into the CLI:

1. `ash run <file>` routes through `ModuleResolver`
2. `ash check <file>` routes through `ModuleResolver`
3. `ASH_LIBRARY_PATH` honored by CLI
4. Import resolution errors surface as user-facing diagnostics
5. `ash trace <file>` works with multi-file workflows

**Files:**
- Modify: `crates/ash-cli/src/commands/run.rs`
- Modify: `crates/ash-cli/src/commands/check.rs` (if exists)
- Create: `crates/ash-cli/tests/run_imports.rs`

**Verification:** CLI end-to-end tests pass; `ash run` executes multi-file workflows.

---

## TASK-659: Module Resolver — Entry Bootstrap Preservation

**Spec:** SPEC-010, existing entry.rs contract
**Track:** A
**Depends on:** TASK-657
**Est. Hours:** 3-4

Verify and harden the entry bootstrap path on the new resolver:

1. Canonical entry workflows (`workflow main() -> Result<(), RuntimeError>`) still work
2. Entry `use` prelude validation still enforces registered runtime modules
3. Entry return-type checking preserved
4. Both ordinary and entry-shaped files can coexist in the same directory tree
5. Regression tests for all existing entry behavior

**Files:**
- Modify: `crates/ash-engine/src/entry.rs`
- Modify: `crates/ash-engine/tests/entry_verification.rs`

**Verification:** All existing entry tests still pass; new multi-file entry tests pass.
