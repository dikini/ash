# TASK-1730: Parse and preserve minimal notation declarations

## Status: 📝 Planned

## Summary

Add a minimal source-preserving AST and parser support for notation declarations from SPEC-095c,
without resolving imported notation, expanding macros, or implementing binder-introducing mixfix.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c §7: Notation declarations

## Dependencies

- 📝 TASK-1729: Reusable surface traversal for expansion diagnostics

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| User-defined notation declarations | SPEC-095c §7 / Phase 168 | Needed carrier and operator-token substrate first | Partly | Parse and preserve local declarations only | Parser tests show declarations in module AST with spans/raw pattern tokens |
| Binder-introducing mixfix | SPEC-095c §7.4 | Needs hygiene/binder model | No | Reject or defer explicitly | Negative parser/diagnostic test or documented unsupported diagnostic |

## Files

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/parse_module.rs` or the current item parser dispatch
- `crates/ash-parser/src/parse_expr.rs` if operator-token parsing is reused
- `crates/ash-parser/tests/task_1730_notation_declaration_parser_ast.rs`

## Requirements

1. Add `NotationDecl` and related fixity/pattern carriers with spans and raw token spelling.
2. Parse minimal declarations for `prefix`, `infixl`, `infixr`, `infix`, `suffix`, and a conservative
   `mixfix` declaration shape if it can be represented honestly.
3. Preserve callable target path syntax without resolving it in this task.
4. Reject or explicitly diagnose unsupported binder-introducing forms.
5. Add AST rendering/debug coverage where downstream tools have exhaustive matches.

## Current state

Phase 168 preserves raw operator tokens in operator sections but has no module-level notation
declaration item.

## Target state

A module can contain parsed notation declarations as surface items. They are preserved for later
resolution and cannot silently lower to Core.

## TDD steps

1. Add parser tests for representative prefix, infix-left, infix-right, infix-nonassoc, and suffix
   declarations.
2. Add a negative test for unsupported binder-style mixfix or malformed patterns.
3. Implement carriers and parser dispatch.
4. Update exhaustive matches in parser consumers and REPL/display code as needed.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1730_notation_declaration_parser_ast
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] Notation declarations parse into durable AST carriers.
  - [ ] Raw operator/pattern spelling and spans are preserved.
  - [ ] Unsupported binder-introducing forms fail closed or are explicitly deferred.
```

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the notation-declaration carrier consumed by TASK-1732.
