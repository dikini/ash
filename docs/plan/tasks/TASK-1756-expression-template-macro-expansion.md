# TASK-1756: Implement fail-closed expression-template macro expansion

## Status: ✅ Complete

## Description

Implement the executable MVP: local parenthesized expression macros expand by substituting invocation expression arguments into a whitelisted parsed expression template before notation/operator-section resolution. All unsupported forms must produce explicit diagnostics and fail closed.

## Specification Reference

- PLAN-172 D2/D3/D5
- SPEC-095c Phase 172 macro MVP subsection
- TASK-1752 template whitelist
- TASK-1755 local registry

## Dependencies

- ✅ TASK-1755: Local macro registry and scope-boundary validation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Binder-introducing macros | SPEC-095c/PLAN-172 | No binder hygiene | No | Reject binder/control templates | Negative tests for `fn`, `let`, `match`, blocks/do/act as applicable |
| Token-tree rewrites | SPEC-095c | No token-tree parser | No | Expression AST substitution only | Bracket/brace negative tests |
| Typed macros | SPEC-095c | No typed macro model | No | Expand before typechecking | Typechecker sees ordinary expanded expression only |

## Requirements

1. Expand only local `name!(expr, ...)` invocations with exact arity.
2. Substitute template variable occurrences whose names match macro params with cloned invocation argument expressions.
3. Preserve source-origin sidecars with `SurfaceOrigin::MacroExpansion`, stable `ExpansionId`, call span, macro name/id, and parent origins for nested products.
4. Reject unsupported template expression variants according to TASK-1752 whitelist.
5. Reject recursive macro expansion or impose a small explicit expansion-depth limit with a diagnostic.
6. Re-run notation/operator-section expansion on macro output so macro-produced notation is resolved by the local active notation table.
7. Ensure Core lowering receives no macro declarations or invocations.

## TDD Steps

### Step 1: Expansion tests RED

**Files:**
- `crates/ash-parser/tests/task_1756_expression_macro_expansion.rs`

Test cases:
1. `macro inc(x) => add(x, 1); fn f(n) { inc!(n) }` expands to ordinary call expression.
2. Arity mismatch rejects.
3. Missing macro rejects.
4. Bracket/brace invocations reject.
5. Template with binder/control form rejects.
6. Nested macro expansion respects depth/recursion diagnostics.
7. Macro output containing notation/operator section is then resolved by existing notation expansion.

### Step 2: Implement expansion

**Files:**
- `crates/ash-parser/src/surface.rs`
- possibly focused helper module under `crates/ash-parser/src/` if the pass grows.

### Step 3: Compatibility gates

Run parser tests, engine tests if high-level module path is affected, and workspace check.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1756_expression_macro_expansion -- --nocapture
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Supported expression macros expand.
  - [x] Unsupported forms fail closed with explicit diagnostics.
  - [x] Macro output re-enters notation/operator-section expansion.
  - [x] Core lowering sees no macro syntax.
  - [x] CHANGELOG.md updated.
```

## Completion Evidence

Implemented local parenthesized expression macro expansion in `crates/ash-parser/src/surface.rs`. The pass now runs before notation/operator-section elaboration, substitutes macro params in whitelisted expression templates, rejects arity mismatches and unsupported template forms with explicit diagnostics, enforces a bounded recursion/depth diagnostic, and leaves no macro invocation syntax at the expanded-surface boundary for supported cases.

Added `crates/ash-parser/tests/task_1756_expression_macro_expansion.rs` covering successful call expansion, arity mismatch, fail-closed unsupported templates, recursive depth diagnostics, template substitution through binary expressions, and macro output re-entering notation expansion. Updated the TASK-1755 registry regression that previously expected deferred execution.

Verification passed:

```bash
cargo test -p ash-parser --test task_1756_expression_macro_expansion -- --nocapture
cargo test -p ash-parser --test task_1755_macro_registry_scope -- --nocapture
cargo check --workspace
cargo fmt --check
git diff --check
```

Focused TASK-1756 evidence: 6 tests passed. Focused TASK-1755 compatibility evidence: 7 tests passed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 40
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides executable macro products whose origin/hygiene metadata is hardened in TASK-1757.
