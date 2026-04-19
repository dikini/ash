# PLAN: `builtin fn` Declaration Form

> **For Hermes:** Use `subagent-driven-development` and `ash-phase-implementation` skills to execute this plan task-by-task.

**Goal:** Add `builtin fn` as a first-class declaration form for pure runtime-provided functions, migrate all magic evaluator builtins to proper `.ash` declarations, and convert regex from a capability provider to a pure builtin.

**Architecture:** Seven tracks: (A) parser and surface AST foundation, (B) module loader and typechecker, (C) runtime dispatch, (D1) stdlib monomorphic builtin migration, (D2) stdlib polymorphic list ops (deferred), (E) regex capability-to-builtin migration, (F) cleanup and verification.

**Tech Stack:** Rust, `ash-parser` (winnow), `ash-core` (AST), `ash-typeck`, `ash-interp` (evaluator), `ash-engine` (module loader, providers), `std/src/*.ash`.

**Design Reference:** [DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN](../design/DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN.md)
**Spec Reference:** [SPEC-BUILTIN-FN](../spec/SPEC-BUILTIN-FN.md)

---

## Phase Overview

| Track | Tasks | Deliverable |
|-------|-------|-------------|
| A | TASK-614 – TASK-617 | Parser and surface AST substrate for `builtin fn` declarations |
| B | TASK-618 – TASK-620 | Module loader, typechecker, and import resolution |
| C | TASK-621 – TASK-622 | Runtime builtin dispatch mechanism |
| D1 | TASK-623, TASK-626 | Stdlib `.ash` declarations for strictly monomorphic builtins (string ops, record ops) |
| D1.5 | TASK-625-DEFERRED | Type predicate builtins (blocked on ad-hoc polymorphism) |
| D2 | TASK-624-DEFERRED | Stdlib `.ash` declarations for polymorphic list ops (deferred) |
| E | TASK-627 → TASK-628 → TASK-630 → TASK-629 | Regex migration: capability → builtin |
| F | TASK-631A, TASK-631B, TASK-632, TASK-633 | Cleanup, changelog, verification |

---

## Track A: Parser and Surface AST (Foundation)

These tasks build the grammar and AST support. No runtime behavior changes yet.

---

### TASK-614: Add `builtin` Keyword and `BuiltinFnDef` Surface AST Variant

**Objective:** Add `builtin` as a reserved keyword and a new `Definition::BuiltinFn` variant in the surface AST (`ash-parser`). Lowering to `ash-core` IR is handled by TASK-616.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs` (add `BuiltinFnDef` struct and `Definition::BuiltinFn` variant to the surface `Definition` enum at line 76)
- Modify: `crates/ash-parser/src/keywords.rs` or equivalent (add `builtin` to reserved keywords)

**TDD Steps:**

1. **Red:** Write a test that constructs a `Definition::BuiltinFn` with name, params, return type, no body, and verifies the fields are populated. This test should fail to compile because the variant doesn't exist yet.
2. **Green:** Add `BuiltinFnDef { visibility, name, type_params, params, return_type, span }` to the surface AST in `surface.rs`. Add `Definition::BuiltinFn(BuiltinFnDef)` variant. The key difference from `FnDef` is: no `body` field.
3. **Verify:** `cargo test -p ash-parser` passes.

**Estimated hours:** 2-3

---

### TASK-615: Parse `builtin fn` Declarations

**Objective:** Extend the parser to recognize `builtin fn` declarations: semicolon-terminated, no body, no braces.

**Files:**
- Modify: `crates/ash-parser/src/parse_module.rs` (add `builtin fn` parsing branch)
- Create: `crates/ash-parser/tests/builtin_fn_parser.rs`

**TDD Steps:**

1. **Red:** Write parser tests:
   ```
   // Accepts valid form
   "builtin fn foo(x: Int) -> Int;"
   "pub builtin fn bar(s: String) -> Bool;"
   
   // Rejects body form
   "builtin fn foo(x: Int) -> Int { x }"  // should error: unexpected '{'
   
   // Rejects missing semicolon
   "builtin fn foo(x: Int) -> Int"  // should error: expected ';'
   ```

2. **Green:** Implement parsing in `parse_module.rs`. The parsing logic:
   - After visibility modifier, check for `builtin` keyword before `fn`
   - Parse name, type params, params, return type same as `pub fn`
   - Expect `;` terminator (not `{` block)
   - Emit `Definition::BuiltinFn(BuiltinFnDef { ... })`
3. **Verify:** All parser tests pass. `cargo test -p ash-parser` green.

**Dependencies:** TASK-614

**Estimated hours:** 3-4

---

### TASK-616: Lower `BuiltinFnDef` to IR

**Objective:** Extend the lowering pass to handle `BuiltinFnDef`. Since it has no body, lowering produces a type-only registration (no expression IR).

**Files:**
- Modify: `crates/ash-parser/src/lower.rs` (handle `Definition::BuiltinFn`)
- Modify: `crates/ash-core/src/ast.rs` (add IR representation for bodyless builtins if needed)
- Create/modify: tests for lowering round-trip

**TDD Steps:**

1. **Red:** Test that lowering a `BuiltinFnDef` produces an IR callable with a `Builtin` marker and no body expression.
2. **Green:** Add lowering branch for `Definition::BuiltinFn`. The lowered form is a callable entry with the declared type signature and a flag/marker indicating "builtin -- dispatch at runtime, no body to lower."
3. **Verify:** `cargo test -p ash-parser` passes.

**Dependencies:** TASK-614

**Estimated hours:** 2-3

---

### TASK-617: Module-Level Snippet Extraction for `builtin fn` in `.ash` Files

**Objective:** Ensure `collect_module_exports` in the module loader recognizes `builtin fn` snippets. `builtin fn` is semicolon-terminated, so it uses `extract_semicolon_snippets` (like `pub type`), not `extract_braced_snippets` (like `pub fn`).

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs` (add `builtin fn` to snippet extraction in `collect_module_exports` at line ~430-479)
- Create: integration test parsing a multi-declaration `.ash` file with mixed `pub fn`, `builtin fn`, and `pub type`

**TDD Steps:**

1. **Red:** Test that a source file containing `builtin fn` declarations is recognized by `collect_module_exports` through `extract_semicolon_snippets`.
2. **Green:** Add `builtin fn` to `extract_semicolon_snippets` alongside `pub type`. Each extracted snippet is parsed individually -- do not reuse `parse_fn_definition` (which only handles `[pub] fn ... { body }`, not `builtin fn ... ;`).
3. **Verify:** `cargo test -p ash-parser` green.

**Dependencies:** TASK-615, TASK-616

**Estimated hours:** 1-2

---

## Track B: Module Loader and Typechecker

These tasks make `builtin fn` declarations visible to the module system and type system.

---

### TASK-618: Module Loader Registers `builtin fn` as Callable Exports

**Objective:** Extend `module_loader.rs` to recognize `builtin fn` snippets (extracted via `extract_semicolon_snippets`) and register them as exported callables with a builtin marker.

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs` (add `builtin fn` snippet extraction and registration)
- Create: `crates/ash-engine/tests/builtin_fn_module_loading.rs`

**Design note -- InlineCallable representation:** Current `InlineCallable` has `body: Expr` (required). This task must introduce a `CallableKind` enum (`CallableKind::User { body: Expr }` vs `CallableKind::Builtin`) to support bodyless builtins, avoiding `Option<Expr>` unwrap risks.

**TDD Steps:**

1. **Red:** Create a test `.ash` file with `builtin fn` declarations. Attempt to load it and import the builtins. Expect failure (not recognized yet).
2. **Green:**
   - Add `extract_semicolon_snippets` for `builtin fn` in `collect_module_exports` (semicolon-terminated like `pub type`, not braced like `pub fn`).
   - Parse each snippet through the parser -- do not reuse `parse_fn_definition` (only handles `[pub] fn ... { body }`).
   - Introduce `CallableKind` enum. Change `InlineCallable` to carry `kind: CallableKind`.
   - Register bodyless builtins as `InlineCallable { kind: CallableKind::Builtin, ... }`.
   - Export to callers via the same mechanism as `pub fn`.

3. **Verify:** Test file loads, exports resolve, `use module::{name}` works.

**Dependencies:** TASK-617

**Estimated hours:** 3-4

---

### TASK-619: Typechecker Resolves `builtin fn` Type Signatures

**Objective:** Extend the typechecker to handle `builtin fn` declarations. They type identically to `pub fn` (pure, `Type::Fn(params, ret)`).

**Files:**
- Modify: `crates/ash-typeck/src/lib.rs` or relevant type-checking module
- Modify: `crates/ash-typeck/src/type_env.rs` (if needed for builtin fn registration)
- Create: `crates/ash-typeck/tests/builtin_fn_typecheck.rs`

**TDD Steps:**

1. **Red:** Test that a call to a `builtin fn` (e.g., `string::concat("a", "b")`) typechecks correctly using the declared signature.
2. **Green:** Extend typechecker to recognize `builtin fn` definitions. The type is `Type::Fn(params, ret)` -- identical to `pub fn`. No effect annotation.
3. **Verify:** Typechecker tests pass. Calls to builtin fns typecheck correctly.

**Dependencies:** TASK-618

**Estimated hours:** 2-3

---

### TASK-620: End-to-End Import Resolution for `builtin fn`

**Objective:** Verify the full pipeline: `.ash` file declares `builtin fn`, another file imports it via `use`, typechecker resolves the call, and the import succeeds at module-load time.

**Files:**
- Create: `crates/ash-engine/tests/builtin_fn_e2e_import.rs`

**TDD Steps:**

1. **Red:** Create two test `.ash` files -- one declaring `builtin fn add(a: Int, b: Int) -> Int;` and another that imports and calls it. Expect the import to resolve but the call to fail at runtime (no Rust implementation yet).
2. **Green:** Verify import resolution and typechecking work. Runtime dispatch failure is expected and handled by Track C.
3. **Verify:** Import resolves, typecheck passes, runtime produces "builtin not implemented" error.

**Dependencies:** TASK-618, TASK-619

**Estimated hours:** 2-3

---

## Track C: Runtime Dispatch

---

### TASK-621: Runtime Builtin Dispatch Table

**Objective:** Add a builtin dispatch mechanism to the evaluator. When a `builtin fn` call reaches the evaluator, dispatch to the correct Rust implementation by qualified name.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs` (add builtin dispatch path)
- Create: `crates/ash-interp/tests/builtin_dispatch.rs`

**TDD Steps:**

1. **Red:** Write a test that exercises a known builtin (e.g., `string::concat("hello ", "world")`) through the new dispatch path. Expect it to work since `eval_function_call` already handles it.
2. **Green:** Add a dispatch path that recognizes "this is a builtin fn call" (from the `CallableKind::Builtin` marker) and routes to the existing `eval_function_call` match arms. The existing hardcoded dispatch arms become the implementation for builtin fn declarations.
3. **Verify:** `cargo test -p ash-interp` green. Existing behavior preserved.

**Dependencies:** TASK-618

**Estimated hours:** 3-4

---

### TASK-622: Clear Error on Unknown Builtin

**Objective:** When a `builtin fn` is called but has no Rust implementation in the dispatch table, produce a clear error message.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs` (error path)
- Create: test for unknown-builtin error

**TDD Steps:**

1. **Red:** Declare a `builtin fn mystery(x: Int) -> Int;` in a test `.ash` file, import it, call it. Expect a clear error: "builtin function 'module::mystery' not implemented in runtime".
2. **Green:** Add fallback error path in the builtin dispatch when the qualified name is not found in the implementation table.
3. **Verify:** Test passes. Error message is clear and actionable.

**Dependencies:** TASK-621

**Estimated hours:** 1-2

---

## Track D1: Stdlib Strictly Monomorphic Pure-Builtin Migration

These tasks create `.ash` declarations for currently magic pure builtins with
strictly monomorphic type signatures (all parameters and return types are
concrete -- no type variables). Each task is independent.

> **Backward-compatibility note:** The current evaluator supports dual dispatch
> for `starts_with` and `ends_with` (both `string::starts_with` and bare
> `starts_with`). After Track D1, only the qualified form is supported. This is
> a **breaking change**: Ash code calling bare `starts_with(...)` or
> `ends_with(...)` must be updated to `string::starts_with(...)` etc. All other
> currently unqualified builtins (type predicates, list ops, record ops) remain
> available as-is until their respective deferred tracks unblock. See
> SPEC-BUILTIN-FN Section 9.5 for the full inventory.

---

### TASK-623: Create `std/src/string.ash` with Builtin Declarations

**Objective:** Declare the four string builtins that are currently hardcoded in the evaluator.

**Files:**
- Create: `std/src/string.ash`
- Modify: `std/src/lib.ash` or `std/src/lib.rs` (add `pub mod string` if needed)

**Declaration:**
```ash
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;
```

**TDD Steps:**

1. **Red:** Create `string.ash`. Verify it parses and the module loader exports the four functions.
2. **Green:** Wire `pub mod string` into the stdlib. Verify `use string::{concat}` resolves.
3. **Verify:** `cargo test -p ash-engine --test builtin_fn_e2e_import`, `cargo test -p ash-engine --test regex_import_limitation`, and new string-specific tests pass.

**Dependencies:** TASK-617, TASK-621

**Estimated hours:** 2-3

---

### TASK-626: Declare Record Operation Builtins

**Objective:** Declare `keys`, `values`, `record` as builtins.

**Files:**
- Create: `std/src/record.ash` (or add to prelude)
- Modify: type environment registration

**TDD Steps:** Same pattern as TASK-623.

**Dependencies:** TASK-617, TASK-621

**Estimated hours:** 1-2

---

## Track D1.5: Type Predicate Builtins (BLOCKED on Ad-Hoc Polymorphism)

> Type predicates (`is_int`, `is_string`, etc.) accept any value and return
> `Bool`. They quantify over a type variable `<a>` and therefore require at
> least simple ad-hoc polymorphism in the typechecker. They are NOT monomorphic.
> This track is blocked until that mechanism is designed.

---

### TASK-625-DEFERRED: Declare Type Predicate Builtins

**Objective:** Declare `is_int`, `is_string`, `is_bool`, `is_list`, `is_record`, `is_null` as builtins.

**Why deferred:** These require `pub builtin fn is_int<a>(value: a) -> Bool;`
which quantifies over type parameter `<a>`. The current typechecker has no
mechanism for universally-quantified builtin parameters. This is simpler than
full parametric polymorphism (the type parameter is unused in the return type)
but the mechanism does not exist yet.

**Namespace decision:** Recommend `std/src/type.ash` or keep in prelude.

**Status:** DEFERRED -- blocked on ad-hoc polymorphism for builtin type params.

**Files:**
- Create: `std/src/type.ash` (or add to prelude)
- Modify: type environment registration

**TDD Steps:**

1. **Red:** Test that type predicates resolve through the module system.
2. **Green:** Add declarations, wire into module loader and typechecker.
3. **Verify:** Type predicate calls typecheck and execute correctly.

**Dependencies:** TASK-617, TASK-621, ad-hoc polymorphism for builtin type params

**Estimated hours:** 2-3

---

## Track D2: Stdlib Polymorphic List Ops (DEFERRED)

> Polymorphic builtin semantics (generic type parameters in `builtin fn` signatures such as `List<a>`) are deferred per the design note. Track D2 cannot proceed until generic builtin semantics are designed and implemented.

---

### TASK-624-DEFERRED: Declare List Operation Builtins

**Objective:** Declare `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` as builtins. These require generic signatures like `List<a>`, which depend on deferred generic builtin semantics.

**Design decision:** These are currently unqualified builtins (called as `len(xs)` not `list::len(xs)`). Recommend keeping them unqualified in `std/src/prelude.ash` for backward compatibility, with a future deprecation path to `std/src/list.ash`.

**Status:** DEFERRED -- blocked on generic builtin semantics.

**Files:**
- Modify: `std/src/prelude.ash` (or create `std/src/list.ash`)
- Modify: `crates/ash-typeck/src/type_env.rs` (migrate `add_builtin_functions` entries for list ops)

**Dependencies:** TASK-617, TASK-621, generic builtin semantics

**Estimated hours:** 3-4

---

## Track E: Regex Capability-to-Builtin Migration

These tasks convert regex from a capability provider to a pure builtin. Strict ordering: TASK-627 → TASK-628 → TASK-630 → TASK-629. TASK-629 must not start until TASK-630 (positive e2e test) passes.

> **Caveat:** The "which capabilities stay as providers" classification is an intended design target, not an audited inventory of current wiring.

---

### TASK-627: Rewrite `std/src/regex.ash` with `builtin fn` Declarations

**Objective:** Replace the broken `pub fn ... { act execute ... }` bodies with `builtin fn` declarations.

**Files:**
- Rewrite: `std/src/regex.ash`

**Before:**
```ash
pub fn find(pattern: String, text: String) -> Option<String> {
    act execute Regex.find with pattern: pattern, text: text
}
```

**After:**
```ash
pub builtin fn find(pattern: String, text: String) -> Option<String>;
pub builtin fn matches(pattern: String, text: String) -> Bool;
pub builtin fn replace(pattern: String, replacement: String, text: String) -> String;
```

**TDD Steps:**

1. **Red:** Verify current `regex.ash` fails to parse/load (existing limitation test).
2. **Green:** Rewrite with `builtin fn` declarations. Verify file parses and exports resolve.
3. **Verify:** `use regex::{find}` succeeds at module-load time.

**Dependencies:** TASK-617

**Estimated hours:** 1-2

---

### TASK-628: Move Regex Dispatch to Evaluator Builtin Table

**Objective:** Move regex function dispatch from the legacy capability path to the evaluator's builtin dispatch table (pure builtin path).

**Files:**
- Modify: `crates/ash-interp/src/eval.rs` (add `regex::find`, `regex::matches`, `regex::replace` to the builtin match arms)
- Modify: `crates/ash-interp/src/eval.rs` (remove any regex references from capability-dispatch path)

**TDD Steps:**

1. **Red:** Write test calling `regex::find("a+", "abc")` through the builtin dispatch path. Expect failure (not yet wired).
2. **Green:** Add `(Some("regex"), "find")` match arm to `eval_function_call` that performs the regex operation directly (using the `regex` crate). Match the existing regex runtime behavior.
3. **Verify:** Regex operations work through the evaluator without any capability provider.

**Dependencies:** TASK-621, TASK-627

**Estimated hours:** 2-3

---

### TASK-630: Positive End-to-End Regex Test

**Objective:** Prove the regex builtin path works end-to-end with honest positive coverage for import, typecheck, evaluator dispatch, and runtime execution.

**Files:**
- Modify: `crates/ash-engine/tests/builtin_fn_e2e_import.rs`
- Rewrite honestly as needed: `crates/ash-engine/tests/regex_import_limitation.rs` (historical filename may remain if contents are no longer limitation-framed)
- Modify: `CHANGELOG.md`, `docs/plan/PLAN-090-SPEC-PROCESSOR.md`, `docs/plan/tasks/TASK-595-std-regex.md`

**TDD Steps:**

1. **Green:** Add or retain a positive test: create temp `.ash` file with `use regex::{find}`, call the imported builtin through the real engine path, verify result.
2. **Verify:** Positive regex e2e coverage passes. Any remaining `regex_import_limitation` target is historical-name-only and no longer claims a broken boundary.

**Dependencies:** TASK-627, TASK-628

**Estimated hours:** 1-2

---

### TASK-629: Delete legacy regex carrier

**Objective:** Remove the legacy regex capability carrier, its engine wiring, its capability-specific tests, and all repository references describing regex as a capability provider.

**Gate:** Must not start until TASK-630 (positive e2e test) passes.

**Migration inventory -- every surface to touch:**

Code (delete or rewrite):
- Delete: `crates/ash-engine/src/providers/regex.rs`
- Modify: `crates/ash-engine/src/providers/mod.rs` (remove legacy regex module export)
- Modify: `crates/ash-engine/src/lib.rs` (remove regex provider wiring)
- Remove obsolete provider-era regex integration coverage or replace it with builtin-path coverage

Documentation (update or remove provider references):
- Modify: `CHANGELOG.md` (update entries at ~303-304 describing regex as provider)
- Modify: `docs/plan/PLAN-090-SPEC-PROCESSOR.md` (update reference at ~271)
- Modify: `docs/plan/tasks/TASK-595-std-regex.md` (reflect provider deletion, update to builtin path)

Verification:
- Grep the entire repository for stale regex-carrier/provider wording and `regex::find.*act` to confirm cleanup.

**Compatibility tie:** TASK-629 deletion is tied to the backward-compatibility
decision in SPEC-BUILTIN-FN Section 9.5. Regex operations have no currently
unqualified form (they are `regex::find` etc.), so no dual-dispatch issue
exists. Deletion is safe once TASK-630 proves the builtin path works.

**TDD Steps:**

1. **Red:** Verify existing regex tests still reference the capability provider path.
2. **Green:** Delete provider, remove wiring, rewrite tests to use the builtin dispatch path. The 12 existing test cases (find, matches, replace, invalid pattern, etc.) should all pass through the evaluator builtin path instead. Update all documentation references.
3. **Verify:** builtin-path regex tests are green. Repo grep over crates/docs returns zero stale regex-carrier references.

**Dependencies:** TASK-630 passes

**Estimated hours:** 2-3

---

## Track F: Cleanup and Verification

---

### TASK-631A: Remove Hardcoded Builtin Type Entries Covered by D1

**Objective:** `add_builtin_functions()` in `type_env.rs` (called from `add_builtin_types()`) seeds type signatures for 13 builtins. After Track D1, string ops and record ops are covered by `.ash` declarations. Remove any covered hardcoded entries that still remain. In the landed implementation this required deleting the string entries; record entries were already absent. Type predicates remain hardcoded until Track D1.5 unblocks.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs` (remove string-op and record-op entries from `add_builtin_functions`)

**TDD Steps:**

1. **Red:** Verify current tests pass with all hardcoded registrations.
2. **Green:** Remove entries for builtins covered by D1 `.ash` declarations (string ops, record ops). Keep type-predicate entries until Track D1.5. Verify type resolution works through the module system.
3. **Verify:** `cargo test -p ash-typeck` green.

**Dependencies:** TASK-623, TASK-626

**Estimated hours:** 1-2

---

### TASK-631B: Remove Remaining Hardcoded Builtin Type Entries

**Objective:** Remove remaining `add_builtin_functions` entries for list ops. If all 13 builtins are now covered, delete `add_builtin_functions` and its call from `add_builtin_types()` entirely. BLOCKED on Track D2.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs` (remove list-op entries, possibly delete `add_builtin_functions` entirely)

**TDD Steps:**

1. **Green:** Remove remaining entries. If all 13 builtins are declaration-covered, delete `add_builtin_functions` entirely.
2. **Verify:** `cargo test -p ash-typeck` green. No hardcoded builtin type registrations remain.

**Dependencies:** TASK-624-DEFERRED, TASK-631A

**Estimated hours:** 2-3

---

### TASK-632: Update CHANGELOG.md and PLAN-INDEX

**Objective:** Reconcile changelog, PLAN-INDEX, and task-file status surfaces so
they honestly reflect completed Track E work and completed TASK-631A while
keeping TASK-631B blocked and TASK-633 pending.

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md` (add new phase or fold into existing phase)

**Estimated hours:** 1

---

### TASK-633: Full Workspace Verification

**Objective:** Run all quality gates.

**Commands:**
```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps
```

**Estimated hours:** 1

---

## Gating Notes

- **Actionable now:** Track A (TASK-614 through TASK-617) -- parser/AST work has no external dependencies
- **Blocked on Track A:** Track B (TASK-618 through TASK-620)
- **Blocked on Track B:** Track C (TASK-621, TASK-622)
- **Partially parallel:** Tracks D1 and E can proceed once Track C is complete; D1 tasks are independent of each other
- **Track E strict ordering:** TASK-627 → TASK-628 → TASK-630 → TASK-629. TASK-629 gated on TASK-630 passing.
- **D2 deferred:** TASK-624-DEFERRED blocked on generic builtin semantics
- **Track F current state:** TASK-631A, TASK-632, and TASK-633 are complete;
  TASK-631B remains blocked on deferred D2 generic builtin semantics
- **Blocked on D2:** TASK-631B blocked on TASK-624-DEFERRED

## Decision Gates

- **D1 (after TASK-617):** Parser accepts `builtin fn`. Hard gate -- if grammar can't parse it cleanly, design needs revisiting.
- **D2 (after TASK-620):** Full import/typecheck pipeline works for bodyless fns.
- **D3 (after TASK-622):** Runtime dispatch works. Feature is functionally complete for a single builtin.
- **D4 (after TASK-630):** Regex e2e test passes. Proven replacement for capability provider.
- **D5 (after TASK-629):** Legacy regex carrier deleted. Migration is irreversible.

## Estimated Total

| Track | Tasks | Hours |
|-------|-------|-------|
| A: Parser/AST | 4 | 8-12 |
| B: Module Loader/Typeck | 3 | 7-10 |
| C: Runtime Dispatch | 2 | 4-6 |
| D1: Stdlib Monomorphic | 3 | 5-8 |
| D2: Stdlib Polymorphic (deferred) | 1 | 3-4 |
| E: Regex Migration | 4 | 6-10 |
| F: Cleanup | 4 | 5-7 |
| **Total** | **21** | **38-57** |
