# TASK-1786: Define canonical macro declaration identity and callable identity boundaries

## Status: 📝 Planned

## Description

Define the canonical syntax-phase macro declaration identity model and its separation from callable identity. This task should establish the Rust data model and spec language before use-site resolution is threaded through consumers.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- ✅ TASK-1784: Phase 175 plan packet
- 📝 TASK-1785: Identity surface audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Add a compact macro declaration identity type in the parser/LSP-facing substrate, likely near `crates/ash-parser/src/surface.rs` or the audited owner module.
2. Define equality/ordering/hash behavior suitable for summaries and cache keys.
3. Document how macro declaration identity differs from callable identity and why neither summary identity grants runtime callability.
4. Add unit tests for same-name macro/function separation, alias-distinct imported summaries, and stable same-file identity shape.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Write failing identity model tests

Add tests that cannot pass with name-only identity for same-name macro/function declarations.

### Step 2: Implement minimal identity model

Add the compact identity type and conversion helpers selected by TASK-1785.

### Step 3: Verify non-callability boundary

Add assertions or docs tests proving macro identity is not accepted as callable identity.

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
  - cargo test -p ash-parser macro_identity -- --nocapture
  - cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Identity type implemented and documented
  - [ ] Same-name separation tests pass
  - [ ] No macro identity accepted as callable identity
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.
