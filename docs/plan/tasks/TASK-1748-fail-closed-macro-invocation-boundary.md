# TASK-1748: Add fail-closed macro invocation boundary without macro execution

## Status: ✅ Complete

## Description

Add or audit a durable macro invocation boundary in the parsed surface layer so future macro work has a syntactic landing zone, while ensuring macro invocations cannot lower to Core or execute in Phase 171. This task is about fail-closed representation and diagnostics, not expansion.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1744: Hygiene, origin, and scope audit
- ✅ TASK-1746: Source/generated identifier hygiene fences
- ✅ TASK-1747: Notation and macro scope-table boundaries

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expansion | SPEC-095c §6 | No macro execution model | No | Add representation/fail-closed boundary only | Tests assert rejection before Core |
| Typed macros | SPEC-095c non-goals | No type-level macro contract | No | Keep out of scope | No typed macro syntax accepted |

## Requirements

1. Determine from TASK-1744 whether macro-call syntax already has a parser carrier. If not, add the narrowest carrier needed to preserve token/span shape for future work.
2. Preserve macro invocation spans, delimiter shape, and conservative raw delimited body text if parsed.
3. Reject unresolved macro invocations before Core lowering and before engine/module-loader public export collection accepts a callable body.
4. Diagnostic text must identify macro expansion as unsupported/future rather than reporting a misleading ordinary parse/type error.
5. Do not execute macros, evaluate token trees, or add typed macro APIs.
6. Preserve existing accepted syntax; new macro lookahead must not steal ordinary paths, calls, attributes, or notation.

## TDD Steps

### Step 1: Write RED macro-boundary tests

**Expected files:**
- `crates/ash-parser/tests/task_1748_macro_invocation_boundary.rs`
- `crates/ash-engine/tests/task_1748_macro_invocation_boundary.rs` if high-level module validation is affected

Test cases:
1. Macro invocation shape is parsed/preserved if this task adds the carrier.
2. Macro invocation in a public callable body is rejected before Core/module export acceptance.
3. Macro invocation inside notation-expanded output is rejected with origin context.
4. Ordinary calls and paths with similar spelling still parse as before.

### Step 2: Implement fail-closed carrier or audit-only rejection

**Likely files:**
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_expr.rs`
- `crates/ash-parser/src/lower.rs`
- `crates/ash-engine/src/module_loader.rs`

### Step 3: Verify no macro execution path exists

Search for any new path that interprets macro token trees or expands macro calls and remove it from this phase.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1748_macro_invocation_boundary -- --nocapture
  - cargo test -p ash-engine --test task_1748_macro_invocation_boundary -- --nocapture
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Macro invocation shape is preserved or explicitly rejected at parser boundary.
  - [x] Macro invocation cannot lower to Core.
  - [x] No macro execution or typed macro API was introduced.
```

## Completion Evidence

Added a narrow parsed `MacroInvocation` carrier with delimiter and conservative raw-delimited-body preservation, plus fail-closed parser/lowering/engine/typechecker boundary handling. Macro invocations such as `make_id!(...)`, `make_id![...]`, and `make_id!{...}` parse for diagnostics but are rejected by expanded-surface validation, direct Core lowering, module checking, importable export collection, and typechecker-facing validation. This is not a token-tree parser and does not implement qualified macro paths. Added parser tests in `crates/ash-parser/tests/task_1748_macro_invocation_boundary.rs` and engine tests in `crates/ash-engine/tests/task_1748_macro_invocation_boundary.rs`. No macro execution, token-tree evaluation, or typed macro API was introduced.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides macro-boundary negative cases for TASK-1749.
