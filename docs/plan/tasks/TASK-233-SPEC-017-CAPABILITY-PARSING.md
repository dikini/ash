# TASK-233: SPEC-017 Revision - Capability Definition Parsing

## Status: Ready for Development

## Description

Revise SPEC-017 to add capability definition parsing requirements, enabling users to define capabilities directly in `.ash` source files rather than requiring pre-registration in Rust.

## Background

Currently, capabilities work at runtime via `BehaviourProvider`/`StreamProvider` traits, but the parser cannot parse capability definitions from `.ash` files. The `capability_def` parser in `parse_module.rs` is a stub that skips capability definitions.

The idea space document `todo-examples/definitions/capabilities.md` describes rich capability syntax with:
- Effect types (`observe`, `read`, `analyze`, `act`, `write`, `external`)
- Parameter lists with types
- Return types
- `where` constraints

## Requirements

### 1. Syntax Specification

Add to SPEC-017 a complete BNF grammar for capability definitions:

```bnf
capability_def  ::= "capability" IDENTIFIER ":" effect_type
                    "(" param_list? ")"
                    ("returns" type)?
                    constraint_list?

effect_type     ::= "observe" | "read" | "analyze" | "decide" 
                  | "act" | "write" | "external"
                  | "epistemic" | "deliberative" | "evaluative" | "operational"

constraint_list ::= "where" constraint ("," constraint)*
constraint      ::= expression
```

### 2. Semantic Requirements

Document:
- Effect type mapping to `Effect` enum variants
- Constraint evaluation at capability invocation time
- Type checking for capability parameters and returns
- Capability visibility (module-level vs public)

### 3. Integration Points

Specify how capability definitions integrate with:
- Module system (export/import)
- Type checker (capability type validation)
- Runtime (capability registry population)
- Policies (capability references in policy rules)

## Acceptance Criteria

- [ ] SPEC-017 contains complete capability definition syntax
- [ ] Effect type mapping to `Effect` lattice documented
- [ ] Constraint evaluation semantics specified
- [ ] Integration with module system documented
- [ ] Examples of valid/invalid capability definitions
- [ ] Migration path from pre-registered to parsed capabilities

## Dependencies

- None (specification-only task)

## Related Documents

- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md` (to be revised)
- `todo-examples/definitions/capabilities.md` (idea space)
- `crates/ash-core/src/ast.rs` (Capability struct)
- `crates/ash-parser/src/parse_module.rs` (stub to be filled)

## Notes

This is a specification-only task. Implementation will be TASK-234.

The specification should be backward-compatible: existing pre-registered capabilities continue to work.
