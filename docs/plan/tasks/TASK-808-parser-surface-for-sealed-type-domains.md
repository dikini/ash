# TASK-808: Parser Surface for Sealed Type Domains

## Status: ✅ Complete

## Description

Add the restricted `sealed type domain` declaration surface to the ModuleFile parser path without widening general type-expression syntax.

## Specification Reference

- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)
- [PLAN-107](../PLAN-107-SEALED-TYPE-LEVEL-DOMAINS.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## Dependencies

- [TASK-807](TASK-807-sealed-domain-audit-gate.md)

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Objective

Parse sealed-domain declarations as first-class ModuleFile definitions with source-aware domain, constructor, and field carriers.

## Requirements

1. Add a surface AST carrier for sealed domains, marker constructors, and ordered fields.
2. Wire `sealed type domain` through top-level definition dispatch, inline-module behavior if supported, and unknown-item recovery boundaries.
3. Preserve visibility, declaration spans, constructor spans, field spans when available, and source origin.
4. Restrict field annotations to `Type` or visible domain names in this phase; do not parse arbitrary type-expression shapes as domain-field annotations.
5. Add parser tests for fully public domains, mixed constructor visibilities, recursive domain references, files containing only domains, and rejection of unsupported field-annotation shapes.
6. Do not lower semantically in parser.

## Files

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Create or modify parser helpers under `crates/ash-parser/src/`
- Add tests under `crates/ash-parser/tests/`

## TDD Steps

1. Write failing parser tests for the supported domain syntax and the unsupported field-annotation boundary.
2. Implement the minimal ModuleFile parser and surface-carrier changes.
3. Re-run focused parser tests.
4. Confirm ordinary `type`/workflow/module parsing still works.

## Verification

```
strictness: clean
commands:
  - cargo test -p ash-parser --test task_808_sealed_domain_surface
  - cargo test -p ash-parser
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Focused parser tests pass
  - [ ] Full ash-parser suite passes
  - [ ] Clippy clean
  - [ ] Formatting clean
```

## Notes

Parser-only task. Do not add lowering, summary generation, `type fn` syntax, or general marker-constructor application syntax here.
