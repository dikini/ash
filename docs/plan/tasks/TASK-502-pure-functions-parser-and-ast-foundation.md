# TASK-502: Pure Functions Parser and AST Foundation

## Status: 📝 Planned

## Description

Add the parser and surface-AST foundation required for pure `fn` support: fn definitions,
function-type syntax, block/match/if/panic forms in fn bodies, and entry-point-aware file parsing.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [SPEC-002: Surface Language](../../spec/SPEC-002-SURFACE.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)

## Requirements

1. Add lexer/token support for `fn`, `panic`, and `match`.
2. Add surface AST support for `FnDef`, `Expr::If`, `Expr::Panic`, `Expr::Block`, and qualified
   fn-call syntax.
3. Implement parsing for fn definitions, fn types, one-armed/else-armed `if`, `match`, and panic.
4. Ensure entry-point loading reuses `ModuleFile` parsing rather than reintroducing a second root.

## Dependencies

- [TASK-501](TASK-501-pure-functions-prerequisites-and-parser-model.md)

## Likely Files

- Modify: `crates/ash-parser/src/token.rs`
- Modify: `crates/ash-parser/src/lexer.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: parser tests under `crates/ash-parser/tests/`

## Completion Checklist

- [ ] fn token/lexer support added
- [ ] match token/lexer support added
- [ ] fn AST nodes added
- [ ] fn type syntax parses
- [ ] if/match/panic/block fn-body forms parse correctly
- [ ] entry-point promotion uses `ModuleFile` parsing path
