# TASK-1755: Add local macro registry and scope-boundary validation

## Status: ✅ Complete

## Description

Build the local macro registry used by expansion. Macro declarations are visible only in their declaring module for the MVP. Imports, re-exports, glob imports, and module summaries must not activate macros.

## Specification Reference

- PLAN-172 D4
- SPEC-095c Phase 172 macro MVP subsection
- TASK-1752 audit artifact
- TASK-1754 carriers

## Dependencies

- ✅ TASK-1754: Parsed macro declaration and structured invocation carriers

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported macro activation | PLAN-171/172 | No macro summary carrier | No | Keep rejected | Engine/module-loader negative tests |
| Public macro visibility | MVP syntax allows visibility? | Visibility carrier may parse | No export behavior | Public macro declarations do not create importable exports |
| Duplicate macro names | Registry design | Undefined | Yes | Reject duplicates locally | Parser/expansion diagnostic tests |

## Requirements

1. Add a local macro table construction pass in `crates/ash-parser/src/surface.rs` or a focused helper module.
2. Reject duplicate local macro names with spans.
3. Resolve unqualified invocation names only against local macro declarations.
4. Do not include macro declarations in ordinary callable/type/module export summaries.
5. Ensure imported/re-exported macro declarations do not activate in callers.
6. Add focused parser/engine tests for local visibility, duplicate rejection, missing macro rejection, and import/re-export non-activation.

## TDD Steps

### Step 1: Registry tests RED

**Files:**
- `crates/ash-parser/tests/task_1755_macro_registry_scope.rs`
- `crates/ash-engine/tests/task_1755_macro_registry_scope.rs`

Test cases:
1. Local macro is found by local invocation.
2. Duplicate local macro names reject before expansion.
3. Missing macro invocation remains fail-closed with explicit diagnostic.
4. Imported macro declaration does not activate in caller.
5. `pub macro` if parsed does not become an importable callable export.

### Step 2: Implement local registry

Wire the registry into the expanded-surface validation path but do not expand template bodies yet; recognized invocations may still report a planned unsupported-execution diagnostic until TASK-1756.

### Step 3: High-level validation

Run parser and engine focused tests plus workspace check.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1755_macro_registry_scope -- --nocapture
  - cargo test -p ash-engine --test task_1755_macro_registry_scope -- --nocapture
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Local registry tests pass.
  - [x] Duplicate/missing/imported macro behavior fails closed.
  - [x] Module exports do not include macros.
  - [x] CHANGELOG.md updated.
```

## Completion Evidence

Added `LocalMacroTable` construction to `crates/ash-parser/src/surface.rs`, duplicate-name rejection, local-only invocation validation, explicit unknown/unsupported macro diagnostics, and engine regressions proving `pub macro` declarations are not transported as imported callables and imported macros do not activate in callers.

Verification passed:

```bash
cargo test -p ash-parser --test task_1755_macro_registry_scope -- --nocapture
cargo test -p ash-engine --test task_1755_macro_registry_scope -- --nocapture
cargo check --workspace
cargo fmt --check
git diff --check
```

Focused parser evidence: 7 tests passed. Focused engine evidence: 2 tests passed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides name resolution and scope boundaries for TASK-1756 expansion.
