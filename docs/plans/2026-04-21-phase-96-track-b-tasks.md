# Track B Tasks: Stdlib Builtin Completion (Corrected per Semantic Classification)

**Key correction:** Ash has three distinct stdlib function categories:

1. **`builtin fn`** -- Pure, body in Rust runtime. Need dispatch in eval.rs.
2. **`pub fn` with `act`** -- Effectful capability wrappers. Need provider wiring in interpreter, NOT builtins.
3. **`pub fn` with Ash body** -- Pure Ash-source code. Need module resolver to load + execute.

Track B handles ONLY category 1. Category 2 is part of Track A (engine/interpreter wiring).
Category 3 depends on Track A (module resolver) to work at all.

## TASK-660: Stdlib Builtin Audit (REVISED)

**Track:** B
**Depends on:** Nothing
**Est. Hours:** 1-2

Complete the gap table. Current status:

### Implemented (25/28):
| Module | Function | Dispatch Entry | Match Arm |
|---|---|---|---|
| string | concat | Yes | Yes |
| string | starts_with | Yes | Yes |
| string | ends_with | Yes | Yes |
| string | is_empty | Yes | Yes |
| string | to_upper | Yes (implemented: false) | MISSING |
| string | to_lower | Yes (implemented: false) | MISSING |
| string | trim | Yes (implemented: false) | MISSING |
| regex | find | Yes | Yes |
| regex | matches | Yes | Yes |
| regex | replace | Yes | Yes |
| list | len | Yes | Yes |
| list | head | Yes | Yes |
| list | tail | Yes | Yes |
| list | append | Yes | Yes |
| list | concat | Yes | Yes |
| list | filter | Yes | Yes (closure callback) |
| list | map | Yes | Yes (closure callback) |
| predicate | is_int..is_null (6) | Yes | Yes |
| json | parse, stringify, stringify_pretty | Yes | Yes |
| markdown | parse | Yes | Yes |
| record | keys, values, record | Yes | Yes |
| process | run | Yes | Yes |

### Gap Summary:
- 3 unimplemented: `string::to_upper`, `string::to_lower`, `string::trim`
- 2 need closure support verification: `list::filter`, `list::map`

**Deliverable:** Update NOTE-004-STDLIB-BUILTIN-GAP.md with corrected classification.

---

## TASK-661: Implement Missing String Builtins

**Track:** B
**Depends on:** TASK-660
**Est. Hours:** 1-2

Implement the 3 unimplemented string builtins:

1. `string::to_upper(s: String) -> String` -- flip `implemented: false` to `true`, add match arm
2. `string::to_lower(s: String) -> String` -- same
3. `string::trim(s: String) -> String` -- same

These are trivial -- delegate to Rust stdlib's `str::to_upper()`, `str::to_lower()`, `str::trim()`.

**Files:**
- Modify: `crates/ash-interp/src/eval.rs` (3 dispatch entries + 3 match arms)

---

## TASK-664: Verify list::filter and list::map Closure Callbacks

**Track:** B
**Depends on:** TASK-660
**Est. Hours:** 2-3

`list::filter` and `list::map` are declared as builtins that take closure arguments.
Verify they work correctly:

1. Read the current match arm implementations for `filter` and `map`
2. Test with closure arguments: `filter(fn(x) { x > 3 }, [1, 5, 2, 8])` should yield `[5, 8]`
3. Test with qualified call: `list::map(fn(x) { x * 2 }, [1, 2, 3])` should yield `[2, 4, 6]`
4. If closures don't evaluate correctly in builtin context, diagnose and fix

The key question: when `eval_function_call` receives a `Value::Closure` as an argument,
can the builtin handler call back into `eval_expr` to execute it?

**Files:**
- Modify: `crates/ash-interp/src/eval.rs` (if fixes needed)
- Create: tests for filter/map with closures

---

## TASK-665 (Optional): Extract Builtin Handlers to Separate Files

**Track:** B
**Depends on:** TASK-661
**Est. Hours:** 2-3

If eval.rs exceeds 4000 lines after all changes, extract per-module handler functions
into `crates/ash-interp/src/builtins/` as plain functions (not a trait).

**Priority:** Low. Defer unless eval.rs feels unwieldy.
