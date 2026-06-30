# TASK-1746: Implement source/generated identifier hygiene fences

## Status: ✅ Complete

## Description

Prevent generated identifiers from silently capturing source identifiers, or being captured by them, across expansion boundaries. This task implements the smallest source/generated identifier distinction needed for Phase 171 diagnostics; it does not implement full def-site/call-site macro hygiene.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ TASK-1744: Hygiene, origin, and scope audit
- ✅ TASK-1745: Expansion identity and origin-chain carriers

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full def-site/call-site macro hygiene | SPEC-095c §6 | Macro system absent | No | Implement source/generated capture fence only | Negative capture tests must fail closed |
| Binder-introducing mixfix | SPEC-095c §7.4 | Binder hygiene absent | No | Keep rejected/deferred | Tests must not introduce binder mixfix |

## Requirements

1. Classify identifiers introduced by expansion as generated rather than source-written.
2. Ensure generated identifiers cannot resolve to same-scope source bindings by raw text alone unless explicitly marked call-site-resolved by a future task.
3. Ensure source identifiers cannot accidentally bind generated placeholder names produced by notation/operator-section expansion.
4. Add diagnostics that mention expansion origin when a generated identifier boundary fails.
5. Keep ordinary callable resolution unchanged for source-authored identifiers.
6. Avoid broad typechecker rewrites unless TASK-1744 proves the boundary is already in type checking.

## TDD Steps

### Step 1: Write RED hygiene tests

**Expected file:** `crates/ash-parser/tests/task_1746_generated_identifier_hygiene.rs` or nearest existing parser/engine integration-test location.

Test cases:
1. A generated parameter name from operator-section eta expansion does not collide with a source binding of the same spelling.
2. A generated helper name cannot be referenced from source text by spelling alone.
3. Diagnostics include origin/expansion context when a generated-name boundary is violated.

### Step 2: Implement minimal source/generated distinction

**Likely files:**
- `crates/ash-parser/src/surface.rs`
- expansion helper modules identified by TASK-1744
- any name-resolution or lowering diagnostics that consume generated identifiers

### Step 3: Keep full macro hygiene deferred

Do not introduce def-site/call-site semantics beyond what this task tests.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1746_generated_identifier_hygiene -- --nocapture
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Generated identifiers are distinguishable from source identifiers.
  - [x] Capture/collision cases fail closed or avoid collision.
  - [x] Full macro hygiene remains explicitly deferred.
```

## Completion Evidence

Generated operator-section eta parameters now use non-source-spellable names carrying the expansion identity, e.g. `$ash_generated_section_<id>_<role>`, instead of ordinary `__section_lhs`/`__section_rhs` identifiers. Added `crates/ash-parser/tests/task_1746_generated_identifier_hygiene.rs` proving generated names are not parseable from source text, legacy helper-like source bindings do not capture generated parameters, and generated spelling exposes expansion context. This is a source/generated fence only; full def-site/call-site macro hygiene remains deferred.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides identifier hygiene constraints consumed by TASK-1748 macro boundary and TASK-1749 cross-boundary validation.
