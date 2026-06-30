# TASK-1764: Implement bounded imported/exported macro activation

## Status: ✅ Complete

## Description

Use explicit macro summaries to activate imported/exported macros in downstream modules, with positive execution tests and negative leakage/re-export tests.

## Specification Reference

- [PLAN-173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros](../PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-1760: Phase 173 plan packet (complete)
- ✅ TASK-1761: Macro-system expansion seam audit (complete)
- ✅ TASK-1762: Macro-system spec amendments (complete)
- ✅ TASK-1763: Macro summary carriers (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported macro activation | PLAN-172 non-goals | No macro summary carriers | Partially: local macro MVP exists | Implement in Phase 173 via explicit summaries | Positive import/export tests plus negative callable leakage tests |
| Token-tree / bracket / brace macro parsing | PLAN-172 non-goals | Raw carriers only, no token-tree parser | Partially: macro invocation carriers exist | Implement delimiter-preserving carriers before execution | Parser span/delimiter tests and fail-closed unsupported-shape tests |
| Binder-introducing macros | PLAN-172 non-goals | No binder hygiene metadata | Partially: origin chains and generated-name fences exist | Implement only after binder hygiene metadata model | Capture-resistance tests in both directions |
| Typed macro checking / inference | PLAN-172 non-goals | No typed macro summaries | No | Implement after spec and typed signature carriers | Typecheck diagnostics before expansion/Core acceptance |

## Requirements

### Functional Requirements

- [x] Imported public macro can expand at an authorized call site
- [x] Private/non-exported macros remain inaccessible
- [x] Callable imports cannot accidentally activate macro syntax

### Property Requirements

- Macro metadata must remain syntax-phase metadata and must not grant rows, authority, contracts, failures, proof evidence, or runtime provider effects.
- Unsupported or ambiguous macro shapes must fail before Core lowering and before public export acceptance.
- Positive visibility claims require matching negative leakage tests.

## TDD Steps

### Step 1: Inspect current state / write failing evidence

**Files:**
- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-engine/tests/task_1764_imported_exported_macro_activation.rs`
- `crates/ash-parser/tests/task_1764_macro_import_scope.rs`

Current state must be measured from live code before editing. For implementation tasks, write failing parser/engine/typeck tests that demonstrate the exact missing behavior or boundary leak.

### Step 2: Implement or document the minimal scoped change

Keep the task scoped to its listed deliverable. Do not pull later Phase 173 tasks forward unless this task explicitly owns their carriers and tests.

### Step 3: Integrate downstream consumers

Update parser, engine/module-loader, typechecker, lowering, and LSP-facing consumers only as required by this task's public carrier changes.

### Step 4: Verify

Run the commands listed below and record exact evidence in this task file before marking it complete.

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
  - cargo test -p ash-parser
  - cargo test -p ash-typeck
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - git diff --check
checklist:
- [x] Imported public macro can expand at an authorized call site
- [x] Private/non-exported macros remain inaccessible
- [x] Callable imports cannot accidentally activate macro syntax
```

## Completion Evidence

- Added `expand_surface_module_with_imported_macros` and explicit imported `LocalMacroEntry` table insertion in `crates/ash-parser/src/surface.rs`.
- Added engine/module-loader macro activation gated by `MacroSummary` plus internal AST template exports; callable imports with the same spelling do not activate macro syntax.
- Added cycle-safe imported macro collection that reuses the module-loader cache/visiting set so macro activation does not reintroduce import recursion.
- Added regressions in `crates/ash-engine/tests/task_1764_imported_exported_macro_activation.rs` for named imports, aliases, private macro non-leakage, and callable non-activation.
- Verification:
  - `cargo test -p ash-engine --test task_1764_imported_exported_macro_activation -- --nocapture`
  - `cargo test -p ash-engine --test module_file_check_tests constrained_public_interface_import_seeding_does_not_recurse_forever_on_cycles -- --nocapture`
  - `cargo test -p ash-parser`
  - `cargo test -p ash-typeck`
  - `cargo test -p ash-engine`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`

## Notes

Keep Phase 173 conservative. If this task discovers that its scope requires a broader macro system than specified, stop and patch PLAN-173/TASK-1761 with a split recommendation before implementation continues.
