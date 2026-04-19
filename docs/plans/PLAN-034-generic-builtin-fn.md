# PLAN-034: Generic Builtin fn Declarations

**Goal:** Enable `builtin fn` declarations with generic type parameters, unblocking list operations (Track D2) and type predicates (Track D1.5).

**Architecture:** Three tracks: (A) type-system freshening, (B) list ops stdlib, (C) type predicates stdlib. Track A is a prerequisite for B and C.

**Spec Reference:** [SPEC-034](../spec/SPEC-034-generic-builtin-fn.md)

---

## Track A: Type-System Freshening (Prerequisite)

The typechecker's `instantiate_fn_call` does not freshen type variables at call
sites. Currently, `add_builtin_functions()` binds `len` to `Type::Fn([List<Var(N)>], Int)`
with a single shared `Var(N)`. Two calls with different argument types would
incorrectly unify against the same variable.

**However:** `add_builtin_functions` creates a *single fresh `TypeVar`* at
`TypeEnv` construction time, not at each call site. This works because each
`TypeEnv` instance is created per-compilation-unit and `instantiate_fn_call`
starts with an empty substitution. The first call binds `Var(N)`, and the
substitution is local to that call's `CheckResult`.

**Re-analysis needed:** Verify that `check_expr`'s call handling does not
accumulate substitutions across calls. If each call gets its own `Substitution`,
the current approach is safe and no freshening is needed. If substitutions
accumulate, freshening is required.

---

### TASK-634: Audit Type-Variable Scoping in Call Resolution

**Objective:** Determine whether `instantiate_fn_call` needs freshening for
polymorphic builtin fn calls.

**TDD Steps:**
1. **Red:** Write a typecheck test that calls the same polymorphic builtin with
   two different argument types in sequence:
   ```ash
   let x = len([1, 2, 3]);      -- Var(N) ~ Int
   let y = len(["a", "b"]);     -- Var(N) ~ String (must not conflict)
   ```
   If this test passes without freshening, Track A is a no-op.

2. **Green:** If the test fails, add a `freshen` method to `Type` that replaces
   all bound type variables with fresh ones, and call it in `instantiate_fn_call`
   (or in `check_expr` before calling it).

3. **Verify:** Test passes. Existing typecheck tests unaffected.

**Dependencies:** None
**Estimated hours:** 1-2

---

### TASK-635: Add `freshen` to Type (if needed)

**Objective:** Add `Type::freshen(&self, mapping: &mut HashMap<TypeVar, TypeVar>) -> Type`
that replaces all `Type::Var(v)` with fresh variables via a memo map.

**Only implement if TASK-634 audit shows freshening is needed.**

**TDD Steps:**
1. **Red:** Test that `Type::Fn([List<Var(0)>], Var(0)).freshen()` produces
   `Type::Fn([List<Var(N)>], Var(N))` with the same `N` for both occurrences.
2. **Green:** Implement `freshen` using a `HashMap<TypeVar, TypeVar>` memo.
3. **Verify:** Unit test passes.

**Dependencies:** TASK-634
**Estimated hours:** 1-2

---

## Track B: List Operations Stdlib (Track D2)

---

### TASK-636: Create `std/src/list.ash` with Builtin Declarations

**Objective:** Declare the seven list builtins as generic builtin fn.

**Files:**
- Create: `std/src/list.ash`
- Modify: stdlib module registration (add `pub mod list`)

**Declaration:**
```ash
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, elem: a) -> List<a>;
pub builtin fn concat<a>(left: List<a>, right: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, transform: Fn(a) -> b) -> List<b>;
```

**TDD Steps:**
1. **Red:** Create `list.ash`. Verify it parses and module loader exports.
2. **Green:** Wire into stdlib. Verify import resolution.
3. **Verify:** Module loader tests pass.

**Dependencies:** TASK-634
**Estimated hours:** 2-3

---

### TASK-637: Add List Ops to Builtin Dispatch Table

**Objective:** Register qualified list ops (`list::len`, `list::head`, etc.)
in the builtin dispatch table.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs`

**TDD Steps:**
1. **Red:** Test that `list::len([1,2,3])` dispatches correctly via the table.
2. **Green:** Add entries to `builtin_dispatch_table()`.
3. **Verify:** Dispatch tests pass.

**Dependencies:** TASK-636
**Estimated hours:** 1-2

---

### TASK-638: Typecheck List Ops Through .ash Declarations

**Objective:** Verify that list ops typecheck correctly via their .ash
declarations, including polymorphic instantiation.

**TDD Steps:**
1. **Red:** Test that `len([1, 2, 3])` typechecks as `Int`.
2. **Red:** Test that `map([1, 2], |x| => x + 1)` typechecks as `List<Int>`.
3. **Green:** Wire typecheck resolution.
4. **Verify:** Typecheck tests pass.

**Dependencies:** TASK-636
**Estimated hours:** 2-3

---

### TASK-639: End-to-End List Ops Verification

**Objective:** Full integration test: parse, typecheck, evaluate list ops
through the .ash declaration path.

**TDD Steps:**
1. **Red:** Test that `list::len([10, 20, 30])` evaluates to `3`.
2. **Red:** Test that `list::map([1, 2, 3], |x| => x * 2)` evaluates to `[2, 4, 6]`.
3. **Green:** Fix any integration gaps.
4. **Verify:** All tests pass.

**Dependencies:** TASK-637, TASK-638
**Estimated hours:** 1-2

---

## Track C: Type Predicate Stdlib (Track D1.5)

---

### TASK-640: Create `std/src/predicate.ash` with Builtin Declarations

**Objective:** Declare the six type predicate builtins.

**Files:**
- Create: `std/src/predicate.ash`
- Modify: stdlib module registration

**Declaration:**
```ash
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
```

**TDD Steps:**
1. **Red:** Create `predicate.ash`. Verify parsing and module exports.
2. **Green:** Wire into stdlib.
3. **Verify:** Module loader tests pass.

**Dependencies:** TASK-634
**Estimated hours:** 1-2

---

### TASK-641: Add Type Predicates to Builtin Dispatch Table

**Objective:** Register qualified predicates in dispatch table.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs`

**TDD Steps:**
1. **Red:** Test `predicate::is_int(42)` returns `true`.
2. **Green:** Add entries.
3. **Verify:** Tests pass.

**Dependencies:** TASK-640
**Estimated hours:** 1

---

### TASK-642: End-to-End Type Predicate Verification

**Objective:** Integration test for type predicates.

**TDD Steps:**
1. **Red:** Test `is_int(42)` → `true`, `is_int("hi")` → `false`.
2. **Green:** Fix gaps.
3. **Verify:** All pass.

**Dependencies:** TASK-641
**Estimated hours:** 1

---

## Track D: Cleanup

---

### TASK-643: Delete `add_builtin_functions()`

**Objective:** Remove the hardcoded type-env registrations for list ops now
covered by `.ash` declarations. This is the unblocked portion of TASK-631B.

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs` (remove `add_builtin_functions`)

**TDD Steps:**
1. **Red:** Verify all list ops typecheck through .ash declarations.
2. **Green:** Delete `add_builtin_functions()`. Update `add_builtin_types()`.
3. **Verify:** `cargo test --all` passes.

**Dependencies:** TASK-639, TASK-642
**Estimated hours:** 1

---

### TASK-644: Update CHANGELOG and PLAN-INDEX

**Objective:** Document completion.

**Dependencies:** TASK-643
**Estimated hours:** 0.5

---

## Task Summary

|| Task | Description | Track | Est. Hours | Dependencies ||
||------|-------------|-------|------------|-------------||
|| TASK-634 | Audit type-variable scoping in call resolution | A | 1-2 | — ||
|| TASK-635 | Add `freshen` to Type (if needed) | A | 1-2 | TASK-634 ||
|| TASK-636 | Create `std/src/list.ash` | B | 2-3 | TASK-634 ||
|| TASK-637 | Add list ops to dispatch table | B | 1-2 | TASK-636 ||
|| TASK-638 | Typecheck list ops through .ash declarations | B | 2-3 | TASK-636 ||
|| TASK-639 | End-to-end list ops verification | B | 1-2 | TASK-637, TASK-638 ||
|| TASK-640 | Create `std/src/predicate.ash` | C | 1-2 | TASK-634 ||
|| TASK-641 | Add type predicates to dispatch table | C | 1 | TASK-640 ||
|| TASK-642 | End-to-end type predicate verification | C | 1 | TASK-641 ||
|| TASK-643 | Delete `add_builtin_functions()` | D | 1 | TASK-639, TASK-642 ||
|| TASK-644 | Update CHANGELOG and PLAN-INDEX | D | 0.5 | TASK-643 ||
