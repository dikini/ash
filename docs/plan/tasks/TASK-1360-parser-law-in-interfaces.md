# TASK-1360: Parser — `law` keyword in interfaces

## Status: 📝 Planned

## Description

Extend `ash-parser` to accept `law` declarations inside interface bodies.

## Requirements

1. Add `LawDef` AST node to `surface.rs`
2. Add `laws: Vec<LawDef>` field to `InterfaceDef`
3. Add `law` parsing rule to `parse_module.rs` (inside `parse_interface_definition`)
4. Parser accepts `law` declarations with:
   - Name identifier
   - Parameter list (with types)
   - Optional `where` constraints
   - Proposition expression after `:`
5. No regressions in existing parser tests

## Files

- Modify: `crates/ash-parser/src/surface.rs` — add `LawDef` struct and `laws` field to `InterfaceDef`
- Modify: `crates/ash-parser/src/parse_module.rs` — add `law` parsing inside interface bodies
- Modify: `crates/ash-parser/src/lexer.rs` — add `law` as recognized keyword
- Test: `crates/ash-parser/tests/law_syntax.rs`

## Acceptance Criteria

- [ ] `law` parses inside `interface { ... }`
- [ ] Law has name, params, constraints, proposition
- [ ] Parser test passes
- [ ] No regressions in `cargo test -p ash-parser`
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser --all-targets -- -D warnings` clean

## Verification

```bash
cargo test -p ash-parser parse_law_in_interface -- --nocapture
cargo test -p ash-parser
cargo fmt --check
cargo clippy -p ash-parser --all-targets -- -D warnings
```

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)
- [SPEC-078](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
