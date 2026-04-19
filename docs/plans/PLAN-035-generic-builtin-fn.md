# PLAN-035: Generic Builtin fn Declarations

**Goal:** Enable `builtin fn` declarations with generic type parameters, unblocking list operations (Track D2) and type predicates (Track D1.5).

**Architecture:** Four tracks: (A) engine signature propagation, (B) freshening audit, (C) list ops stdlib, (D) type predicates stdlib + cleanup. Track A is the prerequisite for C and D. Track B is an independent audit.

**Spec Reference:** [SPEC-044](../spec/SPEC-044-generic-builtin-fn.md)

**Key insight:** The type system already supports generic builtin fn internally. The blocker is that `ash-engine`'s import path discards type signatures, binding imported builtins as arity-only `Fn(Var, Var, ..., Var) -> Var`.

---

## Track A: Engine Signature Propagation (Prerequisite)

---

### TASK-634: Add `signature` Field to `InlineCallable`

**Objective:** Preserve the full declared type signature of builtin fn callables
through the import pipeline.

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs`

**TDD Steps:**
1. **Red:** Test that an `InlineCallable` parsed from `builtin fn len<a>(list: List<a>) -> Int;`
   carries a `signature` field with the full `BuiltinFnDef` including type params
   and typed parameters.
2. **Green:** Add `signature: Option<ash_parser::surface::BuiltinFnDef>` to
   `InlineCallable`. Populate it in `parse_builtin_fn_callable()`. Ensure clone
   works (or wrap in `Arc`). All existing construction sites for `InlineCallable`
   (user-defined callables, test fixtures) use `signature: None`.
3. **Verify:** `cargo test -p ash-engine` passes. Existing tests unaffected.

**Dependencies:** None
**Estimated hours:** 2-3

---

### TASK-635: Bind Imported Builtin Signatures in `Engine::check()`

**Objective:** Replace the arity-only synthetic type binding with proper declared
signature resolution for imported builtin callables.

**Files:**
- Modify: `crates/ash-engine/src/lib.rs` (all three `check` paths)

**TDD Steps:**
1. **Red:** Test that after importing `builtin fn len<a>(list: List<a>) -> Int;`,
   `Engine::check()` binds `len` as `Type::Fn([List<Var(N)>], Int)` (not
   `Fn([Var(M)], Var(P))`). Test that `len("not a list")` produces a type error.
2. **Green:** In each `Engine::check()` path (lines 531-538, 567-574, and the
   third path around 604), replace the `imported_param_counts` loop with an
   `imported_callables` loop that checks for `signature`. If present, use
   `builtin_fn_signature_type(&type_env, sig)` (re-exported from ash-typeck).
   If absent, fall back to arity-only synthetic.
3. **Verify:** `cargo test -p ash-engine` passes. Existing behavior preserved.

**Dependencies:** TASK-634
**Estimated hours:** 3-4

---

## Track B: Freshening Audit (Independent)

---

### TASK-636: Audit Type-Variable Scoping at Call Sites

**Objective:** Verify that polymorphic builtin calls with different concrete types
at different call sites typecheck correctly without freshening.

**TDD Steps:**
1. **Red:** Write a typecheck test (in ash-typeck) that:
   - Binds `len` as `Type::Fn([List<Var(0)>], Int)` (simulating a generic builtin)
   - Typechecks `len([1, 2, 3])` -- should infer `Var(0) ~ Int`
   - Typechecks `len(["a", "b"])` -- should infer `Var(0) ~ String` independently
   - Both should succeed without conflict
2. **Green:** If the test fails, add `Type::freshen()` and call it in
   `instantiate_fn_call`. If it passes, no freshening needed.
3. **Verify:** Test passes. Document the finding.

**Dependencies:** None
**Estimated hours:** 1-2

---

## Track C: List Operations Stdlib (Track D2)

---

### TASK-637: Create `std/src/list.ash` with Generic Builtin Declarations

**Objective:** Declare the seven list builtins.

**Files:**
- Create: `std/src/list.ash`
- Verify whether std root module export changes are needed

**TDD Steps:**
1. **Red:** Create `list.ash`. Verify it parses and module loader exports.
2. **Green:** Wire into stdlib (verify whether `pub mod list` or equivalent is needed).
3. **Verify:** Module loader test: `use list::{len}` resolves.

**Dependencies:** TASK-635
**Estimated hours:** 2-3

---

### TASK-638: Complete List-Op Qualified Dispatch Wiring

**Objective:** Register qualified list ops as aliases in the dispatch table.

The runtime already supports list ops via unqualified match arms. This task adds
qualified-name entries (`list::len`, `list::head`, etc.) so the imported builtin
dispatch path can route them. This is wiring/consistency work, not new runtime
semantics.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs`

**TDD Steps:**
1. **Red:** Test `list::len([1,2,3])` dispatches correctly.
2. **Green:** Add entries. Primarily wiring/consistency, not new runtime semantics.
3. **Verify:** Dispatch tests pass.

**Dependencies:** TASK-637
**Estimated hours:** 1

---

### TASK-639: Typecheck List Ops Through Imported .ash Declarations

**Objective:** Verify list ops typecheck with correct polymorphic types through
the engine import path (not just direct program typechecking).

**TDD Steps:**
1. **Red:** Test that `len([1, 2, 3])` typechecks as `Int` through the engine.
2. **Red:** Test that `map([1, 2], |x| => x + 1)` typechecks as `List<Int>`.
3. **Red:** Test that `len("not a list")` produces a type error.
4. **Green:** Fix any gaps in the signature propagation path.
5. **Verify:** All pass.

**Dependencies:** TASK-635, TASK-637
**Estimated hours:** 2-3

---

### TASK-640: End-to-End List Ops Verification

**Objective:** Full integration: parse, typecheck, evaluate.

**TDD Steps:**
1. **Red:** `list::len([10, 20, 30])` evaluates to `3`.
2. **Red:** `list::map([1, 2, 3], |x| => x * 2)` evaluates to `[2, 4, 6]`.
3. **Green:** Fix gaps.
4. **Verify:** All pass.

**Dependencies:** TASK-638, TASK-639
**Estimated hours:** 1-2

---

## Track D: Type Predicates + Cleanup

---

### TASK-641: Create `std/src/predicate.ash` with Generic Builtin Declarations

**Objective:** Declare the six type predicate builtins.

**Files:**
- Create: `std/src/predicate.ash`
- Verify whether std root module export changes are needed

**TDD Steps:**
1. **Red:** Create `predicate.ash`. Verify parsing and module exports.
2. **Green:** Wire into stdlib.
3. **Verify:** Module loader tests pass.

**Dependencies:** TASK-635
**Estimated hours:** 1-2

---

### TASK-642: Add Type Predicates to Builtin Dispatch Table + E2E

**Objective:** Register qualified predicates and verify end-to-end.

**TDD Steps:**
1. **Red:** Test `predicate::is_int(42)` returns `true`, `predicate::is_int("hi")` returns `false`.
2. **Green:** Add dispatch entries. Verify e2e.
3. **Verify:** Tests pass.

**Dependencies:** TASK-641
**Estimated hours:** 1-2

---

### TASK-643: Delete `add_builtin_functions()`

**Objective:** Remove hardcoded list-op type registrations now covered by `.ash`
declarations. This is the unblocked portion of TASK-631B.

**Note:** `add_builtin_functions()` currently contains only list ops (type
predicates were already removed by TASK-631A). This task depends only on list
op wiring being complete, not on predicate work.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs`

**TDD Steps:**
1. **Red:** Verify all list ops typecheck through `.ash` declarations.
2. **Green:** Delete `add_builtin_functions()`. Update `add_builtin_types()`.
3. **Verify:** `cargo test --all` passes.

**Dependencies:** TASK-640
**Estimated hours:** 1

---

### TASK-644: Update CHANGELOG and PLAN-INDEX

**Objective:** Document completion.

**Dependencies:** TASK-640, TASK-642, TASK-643
**Estimated hours:** 0.5

---

## Task Summary

|| Task | Description | Track | Est. Hours | Dependencies ||
||------|-------------|-------|------------|-------------||
|| TASK-634 | Add `signature` field to `InlineCallable` | A | 2-3 | — ||
|| TASK-635 | Bind imported builtin signatures in `Engine::check()` | A | 3-4 | TASK-634 ||
|| TASK-636 | Audit type-variable scoping at call sites | B | 1-2 | — ||
|| TASK-637 | Create `std/src/list.ash` | C | 2-3 | TASK-635 ||
|| TASK-638 | Complete list-op qualified dispatch wiring | C | 1 | TASK-637 ||
|| TASK-639 | Typecheck list ops through imported .ash declarations | C | 2-3 | TASK-635, TASK-637 ||
|| TASK-640 | End-to-end list ops verification | C | 1-2 | TASK-638, TASK-639 ||
|| TASK-641 | Create `std/src/predicate.ash` | D | 1-2 | TASK-635 ||
|| TASK-642 | Type predicates dispatch + e2e | D | 1-2 | TASK-641 ||
|| TASK-643 | Delete `add_builtin_functions()` | D | 1 | TASK-640 ||
|| TASK-644 | Update CHANGELOG and PLAN-INDEX | D | 0.5 | TASK-640, TASK-642, TASK-643 ||
