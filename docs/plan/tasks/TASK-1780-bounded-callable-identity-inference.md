# TASK-1780: Implement bounded macro inference through proven callable identities

## Status: ✅ Complete

## Description

Extend bounded macro type inference beyond Phase 173's literal/operator/annotated cases only where TASK-1779 proves a unique callable identity and usable type summary. The implementation must remain fail-closed for ambiguous ordinary calls and must not use macro summaries as callable proofs.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- TASK-1779 callable identity audit
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-1779: Callable identity summary audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Ordinary call expression inference | TASK-1772 | Could fabricate public macro typed summaries from unqualified names | Depends on TASK-1779 | Implement only proven unique identities | Positive unique-call test plus ambiguous-call negatives |
| Callable summaries vs macro summaries | Phase 173 boundary | Macro summaries are syntax-phase only | Yes | Keep distinct | Test macro summary cannot satisfy callable identity proof |

## Requirements

### Functional Requirements

1. Update the macro type inference helper in `crates/ash-parser/src/surface.rs` or the audited owner from TASK-1779.
2. Infer macro result types through ordinary call templates only for callable categories approved by TASK-1779.
3. Preserve Phase 173 behavior for literals, annotated identity, operators, and fully annotated anonymous functions.
4. Add parser tests in `crates/ash-parser/tests/task_1780_callable_identity_inference.rs`.
5. Add engine/module-boundary tests if imported public callable identity is supported.

### Property Requirements

- Inference must be deterministic and unique.
- Ambiguity must not become a best-effort guess.
- Private helpers, macro summaries, unresolved names, and overloaded/interface calls must not produce public macro type summaries unless TASK-1779 explicitly approved them.
- Any inferred summary exported through module boundaries must be checked before expansion output acceptance.

## TDD Steps

### Step 1: Write RED positive tests

Add tests for the exact positive callable identity cases approved by TASK-1779. The initial expected failure should be that no result type is inferred.

### Step 2: Write RED negative tests

Add tests for ambiguous unqualified calls, private helpers, macro-summary confusion, wrong arity, and unresolved names. They should fail if the implementation guesses.

### Step 3: Implement minimal inference

Use only the audited callable identity evidence. Do not add broad resolver behavior or parse raw source snippets.

### Step 4: Verify focused and regression tests

Run TASK-1780 tests plus TASK-1772 and TASK-1771 regressions to ensure the new inference does not weaken previous fail-closed behavior.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1780_callable_identity_inference -- --nocapture
  - cargo test -p ash-parser --test task_1772_macro_type_inference -- --nocapture
  - cargo test -p ash-parser --test task_1771_typed_macro_checking -- --nocapture
  - cargo test -p ash-engine --test task_1772_imported_macro_inference -- --nocapture
  - cargo fmt --check
  - cargo clippy -p ash-parser -p ash-engine --all-targets --all-features -- -D warnings
checklist:
  - [x] Positive callable identity inference cases pass
  - [x] Ambiguous and invalid ordinary calls remain fail-closed
  - [x] Prior typed macro regressions still pass
```

## Dependencies for Next Task

TASK-1781 validates this inference behavior across parser, engine/module-loader, and LSP-facing consumers.

## Completion Evidence

- Bounded macro inference now uses unique local annotated `fn`/`builtin fn` summaries; unresolved, ambiguous, wrong-arity, and module-qualified ordinary calls remain uninferred.
