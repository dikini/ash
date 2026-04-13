# PLAN-027: LLM Stdlb Usability Remediation

## Status: Draft

## References

- **Parent**: [PLAN-025: LLM Standard Library](PLAN-025-LLM-STDLIB.md) (Phase 77, complete)
- **Spec**: [SPEC-029](../../spec/SPEC-029-LLM-STDLIB.md)
- **Amends**: `ash-parser`, `ash-typeck`, `ash-engine`, `std/src/llm/`

## Goal

Resolve the three blockers and two architectural gaps that prevent a real user from building
an end-to-end LLM-powered Ash workflow using only `.ash` code today:

1. 16/23 prompt.ash pub fns silently dropped (enum variant as record field value)
2. Float not a builtin type (Embedding, CompletionParams registration fails)
3. `use llm::Role` parse error from application code (2-segment use path resolution)
4. Missing SPEC-029 functions: `render_template`, `is_final`, `append_response`, `append_tool_result`
5. Three-vertex violation in router.ash and supervised.ash (fn calling workflow)

## Root-Cause Analysis

### Blocker 1: Enum variant as record field value

The parser's `parse_fn_expr` correctly delegates to `expr()` for the general expression
parser, and `expr()` does handle record constructors. Simple cases like
`Pair { a: x, b: x }` parse fine. The failure is specific to using an **enum variant**
(capitalized identifier with no arguments) as a record field value:

```ash
pub fn system(content: String) -> Message {
    Message { role: System, content: content }  -- FAILS
}
```

The parser sees `System` followed by `,` and tries to parse `System` as a new
constructor call (`System { ... }` or `System(...)`). The comma disambiguates too late.
This is the same ambiguity noted in the Ash parser memory: "name {" is parsed as a
record constructor. The restricted parser in `parse_fn_definition` does not have the
disambiguation logic that the workflow-level expression parser has.

**Fix scope**: `crates/ash-parser/src/parse_module.rs` -- `parse_fn_expr` or the
underlying `expr()` call needs lookahead: if `Name` is followed by `,` or `}`, treat it
as a simple identifier (enum variant), not a constructor.

### Blocker 2: Float not builtin

`TypeEnv::add_builtin_types()` registers `Option<T>`, `Result<T, E>`, and `List<T>`.
It does NOT register `Float` (or `Int`, `String`, `Bool`). The first two are registered
implicitly through the type checker's primitive type handling, but `Float` is never added.

Evidence: types.ash fields `embedding: List<Float>` and `temperature: Option<Float>` produce
type registration errors because `Float` is never declared in the type environment.

**Fix scope**: `crates/ash-typeck/src/type_env.rs` -- add `Float` to `add_builtin_types()`
alongside the existing primitives. This is a 5-line change.

### Blocker 3: 2-segment use path resolution

`use llm::Role` is parsed as `UsePath::Simple(["llm", "Role"])`. The module loader
correctly splits this into `module_segments=["llm"]` and `selection=Named("Role")`.
Resolution finds `std/src/llm/mod.ash`. The error occurs during the subsequent loading
of `mod.ash` which contains `pub use` statements that re-export from child modules.

The `pub use` re-export resolution inside `load_ordinary_file` or `collect_module_exports`
fails because `mod.ash` has `pub use types::{Role, Message, ...}` -- a braced import
that triggers loading of `types.ash`. This loading path hits the same parse_fn_definition
limitation for the prompt.ash re-exports.

**Fix scope**: This blocker is partially resolved by fixing Blocker 1 (once fns parse,
re-exports will resolve). The remaining issue is that the error propagation from
`load_ordinary_file` → `collect_module_exports` → child loading masks the real error
as a generic "Parsing Error: ContextError". Improved error context is the fix.

### Gap 4: Missing SPEC-029 functions

SPEC-029 §4 defines `render_template`, `is_final`, `append_response`, `append_tool_result`
as part of the prompt API surface. These are absent from prompt.ash. Adding them requires
parser support (Blocker 1) since they use record constructors.

### Gap 5: Three-vertex violation

SPEC-029 §1.3 defines `fn → fn` (fn never calls workflow/capability). router.ash has
`fn classify_route` that calls `complete()` (a workflow). supervised.ash has
`fn request_approval` that likely has the same pattern.

**Fix**: Restructure these as workflows, or extract the pure logic (prompt construction)
into actual pure fns and move the `complete()` call to the workflow level.

## Task Breakdown

### TASK-545: Add Float as a builtin type

**Estimate**: 1h
**Priority**: Critical (unblocks embeddings)
**Layer**: `ash-typeck`
**Depends on**: Nothing

**Description**: Add `Float` to the builtin type set alongside `Int`, `String`, `Bool`.
Register it in `TypeEnv::add_builtin_types()` so all module files can use `Float` without
declaration.

**TDD Steps**:
1. Red: Write test asserting `Float` resolves in a fresh `TypeEnv::with_builtin_types()`.
2. Red: Write test asserting `Option<Float>` resolves.
3. Red: Write test asserting `List<Float>` resolves.
4. Red: Write test asserting `ash check` on a file with `temperature: Option<Float>` passes.
5. Green: Add `add_float_type()` method to `TypeEnv`, call it from `add_builtin_types()`.
6. Verify: Existing tests pass. `ash check std/src/llm/types.ash` reports 0 Float errors (down from 2).

**Files**:
- Modify: `crates/ash-typeck/src/type_env.rs`
- Add: tests in `crates/ash-typeck/tests/` or `crates/ash-engine/tests/`

---

### TASK-546: Fix enum variant disambiguation in fn expression parser

**Estimate**: 4h
**Priority**: Critical (unblocks 16/23 prompt.ash fns)
**Layer**: `ash-parser`
**Depends on**: Nothing

**Description**: Fix `parse_fn_expr` → `expr()` so that a capitalized identifier followed
by `,` or `}` is parsed as a simple name reference (enum variant), not as the start of a
constructor call. The ambiguity is: `Name { ... }` could be a record constructor or a
name followed by a block. The fix is to add lookahead: if `Name` is immediately followed
by `{` with no `,` inside, it's a constructor. If followed by `,`, `)`, or `}`, it's a
name reference.

This is the same fundamental ambiguity as the match-scrutinee issue noted in the Ash
parser memory. The restricted parser in `parse_fn_body` should apply the same
disambiguation strategy that the workflow parser uses.

**TDD Steps**:
1. Red: Write test: `pub fn f(x: Int) -> Msg { Msg { role: User, text: "hi" } }` parses.
2. Red: Write test: `pub fn f() -> Role { System }` parses (bare enum variant return).
3. Red: Write test: `pub fn f(x: Role) -> Bool { match x { User -> true, _ -> false } }` parses.
4. Red: Write test: `count_pub_fn_snippets(prompt.ash)` returns >= 20 (up from 7).
5. Green: Add lookahead in the expression parser for the `Name {` case: peek ahead for
   `Name { field:` pattern (constructor) vs `Name ,` / `Name }` (variant reference).
6. Green: If ambiguous, prefer variant reference in fn-body context.
7. Verify: All existing parser tests pass. prompt.ash parse count increases significantly.

**Files**:
- Modify: `crates/ash-parser/src/parse_module.rs` (parse_fn_expr / parse_fn_block_expr)
- Possibly modify: `crates/ash-parser/src/parse_utils.rs` or expression parser
- Add: `crates/ash-parser/tests/fn_record_constructor_tests.rs`

**Pitfalls**:
- The `expr()` function is shared between workflow and fn contexts. Changes must not
  break workflow-level constructor parsing.
- Record constructor `Pair { a: 1 }` must still work. Only the bare `Name` (no braces)
  used as a value should be fixed.
- Test with nested constructors: `Outer { inner: Inner { x: 1 } }`.

---

### TASK-547: Fix 2-segment use path and improve import error context

**Estimate**: 2h
**Priority**: High
**Layer**: `ash-engine`
**Depends on**: TASK-546 (partially -- re-export resolution improves once fns parse)

**Description**: Two fixes:
1. Ensure `use llm::Role` resolves correctly from application code. The path parsing and
   module resolution work; the failure is in re-export loading. Once TASK-546 unblocks
   fn parsing, re-exports should work. Verify and add regression tests.
2. Improve error messages: replace generic "Parsing Error: ContextError" with actionable
   context (which module, which pub use, which child failed).

**TDD Steps**:
1. Red: Write test: `use llm::Role; workflow main { done }` resolves from outside std/src/.
2. Red: Write test: `use llm::Message; workflow main { done }` resolves.
3. Red: Write test: `use llm::types::Role; workflow main { done }` still resolves (regression).
4. Red: Write test: `use nonexistent::Foo; workflow main { done }` gives clear error, not
   generic "ContextError".
5. Green: Add error context to `collect_module_exports` and `load_ordinary_file` paths.
6. Verify: All existing import tests pass. New tests pass.

**Files**:
- Modify: `crates/ash-engine/src/module_loader.rs`
- Add: tests in `crates/ash-engine/tests/`

---

### TASK-548: Add missing SPEC-029 prompt functions

**Estimate**: 3h
**Priority**: Medium
**Layer**: `std/src/llm/`
**Depends on**: TASK-546 (record constructor support)

**Description**: Add the four SPEC-029 §4 functions missing from prompt.ash:

- `render_template(template: String, vars: List(Pair)) -> String` -- template variable substitution
- `is_final(response: ChatResponse) -> Bool` -- check if response has finish_reason "stop"
- `append_response(history: List(Message), response: ChatResponse) -> List(Message)` -- append
  assistant message from response to history
- `append_tool_result(history: List(Message), tool_call_id: String, result: String) -> List(Message)`
  -- append tool result message to history

Also verify the existing function signatures match SPEC-029:
- `has_tool_calls` takes `ChatResponse` (not `Message`) per spec

**TDD Steps**:
1. Red: Write test asserting each new function is present in prompt.ash exports.
2. Red: Write test asserting `has_tool_calls` signature matches SPEC-029.
3. Green: Implement each function in prompt.ash using record constructors.
4. Green: Fix `has_tool_calls` parameter type if needed.
5. Verify: `ash check std/src/llm/prompt.ash` passes. `count_pub_fn_snippets` reaches 23+.

**Files**:
- Modify: `std/src/llm/prompt.ash`
- Modify: `std/src/llm/mod.ash` (add re-exports for new functions)
- Add: tests in `crates/ash-engine/tests/llm_stdlib_tests.rs`

---

### TASK-549: Fix three-vertex violations in orchestration modules

**Estimate**: 2h
**Priority**: Medium
**Layer**: `std/src/llm/`
**Depends on**: TASK-546

**Description**: Restructure router.ash and supervised.ash so pure functions don't call
workflows (the `fn → workflow` edge is forbidden by the three-vertex model).

**router.ash**: `fn classify_route` calls `complete()` -- a dispatch workflow.
Fix: Make `classify_route` a workflow, or extract the pure part (prompt construction)
into a fn and move the `complete()` call to the workflow level.

**supervised.ash**: `fn request_approval` may call workflows.
Fix: Same pattern -- separate pure logic from effectful calls.

**TDD Steps**:
1. Red: Write test asserting no fn in router.ash or supervised.ash references `complete`,
   `stream`, `embed`, or any `act` call.
2. Red: Write test asserting router.ash `workflow classify_route` (or equivalent) still
   resolves `llm:chat` capability.
3. Green: Restructure `fn classify_route` → workflow or split into fn + workflow.
4. Green: Restructure supervised.ash similarly.
5. Verify: `ash check` passes on both files. Orchestration patterns still work.

**Files**:
- Modify: `std/src/llm/router.ash`
- Modify: `std/src/llm/supervised.ash`

---

### TASK-550: End-to-end validation and CHANGELOG update

**Estimate**: 2h
**Priority**: Medium
**Layer**: Cross-cutting
**Depends on**: TASK-545, TASK-546, TASK-547, TASK-548, TASK-549

**Description**: Final validation that the LLM stdlib is usable end-to-end:
- All 23+ prompt.ash pub fns parse
- `use llm::Role` resolves from application code
- `ash check std/src/llm/types.ash` reports 0 errors (Float registered)
- A sample LLM workflow (chat + tool use) runs through the engine with mock provider
- Update CHANGELOG.md, PLAN-INDEX.md, task files

**TDD Steps**:
1. Red: Write integration test: parse+check+execute a workflow that imports from llm,
   constructs messages, dispatches chat, and inspects the response.
2. Red: Write test asserting all SPEC-029 sections are substantively covered.
3. Green: Fix any remaining issues discovered during integration.
4. Green: Update CHANGELOG.md, PLAN-INDEX.md.
5. Verify: Full `cargo test` passes. `ash check` passes on all std/src/llm/ files.

**Files**:
- Add: `crates/ash-engine/tests/llm_e2e_usability_tests.rs`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`

## Dependency Graph

```
TASK-545 (Float builtin)     ─────────────────┐
TASK-546 (fn record constructors) ────────────┤
                                               ├──→ TASK-550 (validation)
TASK-547 (use path resolution) ─── depends ──→ │
TASK-548 (missing prompt fns) ── depends ──→   │
TASK-549 (three-vertex fix) ──── depends ──→   │
```

TASK-545 and TASK-546 are independent and can run in parallel.
TASK-547, TASK-548, TASK-549 depend on TASK-546 (need record constructors to work).
TASK-550 depends on all others.

## Estimated Total Effort

| Task | Estimate | Priority |
|------|----------|----------|
| TASK-545 | 1h | Critical |
| TASK-546 | 4h | Critical |
| TASK-547 | 2h | High |
| TASK-548 | 3h | Medium |
| TASK-549 | 2h | Medium |
| TASK-550 | 2h | Medium |
| **Total** | **14h** | |

## Success Criteria

1. `count_pub_fn_snippets(prompt.ash)` >= 23 (up from 7)
2. `ash check std/src/llm/types.ash` reports 0 type errors
3. `use llm::Role; workflow main { done }` resolves from any directory
4. `render_template`, `is_final`, `append_response`, `append_tool_result` present in prompt.ash
5. No fn in llm/ calls a workflow (three-vertex compliance)
6. End-to-end test: build+run an LLM workflow from pure .ash code
