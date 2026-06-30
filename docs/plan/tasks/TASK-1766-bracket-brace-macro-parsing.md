# TASK-1766: Parse bracket and brace macro invocations into structured carriers

## Status: 📝 Planned

## Description

Teach the parser to build structured carriers for `m![...]` and `m!{...}` while keeping unsupported executable shapes fail-closed.

## Specification Reference

- [PLAN-173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros](../PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-1760: Phase 173 plan packet (complete)
- 📝 TASK-1761: Macro-system expansion seam audit (planned)
- 📝 TASK-1762: Macro-system spec amendments (planned)
- 📝 TASK-1765: Token-tree carriers (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported macro activation | PLAN-172 non-goals | No macro summary carriers | Partially: local macro MVP exists | Implement in Phase 173 via explicit summaries | Positive import/export tests plus negative callable leakage tests |
| Token-tree / bracket / brace macro parsing | PLAN-172 non-goals | Raw carriers only, no token-tree parser | Partially: macro invocation carriers exist | Implement delimiter-preserving carriers before execution | Parser span/delimiter tests and fail-closed unsupported-shape tests |
| Binder-introducing macros | PLAN-172 non-goals | No binder hygiene metadata | Partially: origin chains and generated-name fences exist | Implement only after binder hygiene metadata model | Capture-resistance tests in both directions |
| Typed macro checking / inference | PLAN-172 non-goals | No typed macro summaries | No | Implement after spec and typed signature carriers | Typecheck diagnostics before expansion/Core acceptance |

## Requirements

### Functional Requirements

- [ ] Nested delimiter groups parse without losing spans
- [ ] Parenthesized expression macros still use structured Expr args
- [ ] Bracket/brace carriers do not execute until TASK-1767 owns the expansion seam

### Property Requirements

- Macro metadata must remain syntax-phase metadata and must not grant rows, authority, contracts, failures, proof evidence, or runtime provider effects.
- Unsupported or ambiguous macro shapes must fail before Core lowering and before public export acceptance.
- Positive visibility claims require matching negative leakage tests.

## TDD Steps

### Step 1: Inspect current state / write failing evidence

**Files:**
- `crates/ash-parser/src/parse_expr.rs`
- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/tests/task_1766_bracket_brace_macro_parsing.rs`

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
- [ ] Nested delimiter groups parse without losing spans
- [ ] Parenthesized expression macros still use structured Expr args
- [ ] Bracket/brace carriers do not execute until TASK-1767 owns the expansion seam
```

## Notes

Keep Phase 173 conservative. If this task discovers that its scope requires a broader macro system than specified, stop and patch PLAN-173/TASK-1761 with a split recommendation before implementation continues.
