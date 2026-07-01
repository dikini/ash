# TASK-1797: Remove `Value::List` and route all list values through `Cons`/`Nil`

## Status: ✅ Complete

## Description

Remove the legacy `Value::List` runtime variant after TASK-1796 proves every live reference has a migration path. User-facing lists must continue to parse, evaluate, serialize, and pattern-match through `Cons`/`Nil`.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1796 reference classification complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Unknown until audit | Re-evaluate in Phase 176 | Reference classification and removal tests |

## Requirements

### Functional Requirements

1. Remove `Value::List` from `crates/ash-core/src/value.rs`.
2. Route list literal/value construction through the canonical `Cons`/`Nil` representation.
3. Update interpreter, CLI conversion, pattern matching, and tests to use canonical list values.
4. Add a repository assertion that Rust source has no semantic `Value::List` references after the change.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Write RED regression/assertion

Add focused tests for list literal evaluation, stdlib list operations, JSON/list conversion if applicable, and an assertion that `Value::List` is absent from Rust source.

### Step 2: Remove the enum variant

Patch `ash_core::Value` and compile to discover all semantic match arms.

### Step 3: Migrate call sites by owner group

Update core, interpreter, engine, CLI, and tests according to the TASK-1796 map.

### Step 4: Run focused and broad gates

Run affected crate tests first, then workspace check/clippy.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core
  - cargo test -p ash-interp
  - cargo test -p ash-engine
  - cargo test -p ash-cli
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - python3 -c 'from pathlib import Path; bad=[]
for p in Path("crates").rglob("*.rs"):
    s=p.read_text()
    if "Value::List" in s: bad.append(str(p))
assert not bad, bad'
checklist:
  - [x] `Value::List` removed from Rust source
  - [x] Canonical list behavior tests pass
  - [x] Broad gates pass
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

Historical docs/changelog may still mention `Value::List`; the source assertion should scope to Rust code.

Completion evidence: `Value::List` has been removed as a runtime enum variant and the temporary constructor-position compatibility shim has been removed. Runtime and test construction now use `Value::list_from_vec(...)` / `Value::list_nil()`, pattern-position references use semantic accessors, and the repository assertion found no `Value::List` references in Rust source under `crates/`. Verification passed with `cargo fmt --check`, `cargo check --workspace --all-targets`, `cargo test -p ash-core -p ash-interp -p ash-engine -p ash-cli --all-targets`, and `git diff --check`.
