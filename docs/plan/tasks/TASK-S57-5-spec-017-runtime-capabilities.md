# TASK-S57-5: Update SPEC-017 with Runtime-Provided Capability Syntax

## Status: ✅ Complete

## Description

Update SPEC-017 (Capability Integration) with normative syntax for runtime-provided and injected capabilities, resolving the gap between `cap Args` parameter usage in 57B and SPEC-017's earlier focus on declaration-site `capability` syntax.

## Background

Per architectural review, 57B would like to use:

```ash
workflow main(args: cap Args) -> Result<(), RuntimeError> {
  observe Args 0
}
```

**Current SPEC-017 status:** The specification for runtime-provided/injected capabilities is not fully defined. This task establishes the normative syntax and semantics.

**Note:** Current SPEC-017-CAPABILITY-INTEGRATION.md:8-32 focuses on capability definitions, not runtime-injected capability types. This task extends SPEC-017 for the entry point use case.

## Resolved Design

- `cap <Identifier>` is the normative usage-site syntax for a capability-typed parameter.
- `cap` is a type-forming keyword, not a second capability declaration form.
- Capability declarations remain `capability ...` declarations.
- Runtime-provided capabilities are injected at workflow entry, or at another runtime-defined boundary only when that boundary explicitly specifies the same authorization contract.
- Capability invocation remains effect-first and explicit. For `Args`, the canonical form is `observe Args 0`.
- Dotted or method-like forms such as `args.get(0)` are not normative in SPEC-017.

## Requirements

Update SPEC-017 with:

1. **Capability type syntax**: How to write `cap Args` as parameter type
2. **Runtime-provided capabilities**: Special handling for injected capabilities
3. **Capability definition**: Clarify that stdlib still defines `Args` with ordinary `capability` declarations
4. **Usage syntax**: Keep capability invocation in `observe` / `receive` / `set` / `send` forms

## SPEC-017 Sections to Update

### Section: Capability Types

Add:

```
capability_type ::= "cap" identifier

param_type ::= ... | capability_type
```

Example:

```ash
workflow main(args: cap Args)  -- cap Args is the type
```

### Section: Runtime-Injected Capabilities

Add section on capabilities provided by runtime (not user-constructed):

- `cap Args` - CLI arguments
- `cap Stdout` - Standard output
- `cap Stdin` - Standard input

These are "injected" by the runtime, not created by user code.

### Section: Capability Definition in Stdlib

Stdlib declarations continue to use ordinary capability declarations. Example:

```ash
pub capability Args : observe (index: Int) returns Option<String>
```

### Section: Capability Invocation

Capability invocation remains explicit and effect-first. Example:

```ash
workflow main(args: cap Args) -> Result<(), RuntimeError> {
  let argv0 = observe Args 0;
  done;
}
```

Method-like forms such as `args.get(0)` are non-normative.

## Acceptance Criteria

- [x] Capability type syntax (`cap X`) normatively defined
- [x] Runtime-injected capability mechanism specified
- [x] Capability definition syntax for stdlib is clarified
- [x] Usage stays in explicit effect-first capability forms
- [x] Examples show valid capability usage
- [x] Non-normative dotted forms are excluded from SPEC-017
- [x] 57B tasks can use normative syntax

## Related

- SPEC-017: Capability integration
- SPEC-003: Type system (may need updates for capability types)
- MCE-001: Entry point (uses `cap Args`)
- 57B tasks: TASK-361, TASK-362, all workflow signatures

## Est. Hours: 3-4

## Blocking

- TASK-361: Args capability definition
- TASK-362: System supervisor using capabilities
- All workflow signatures using `cap X` parameters

## Completion Summary

SPEC-017 now distinguishes declaration-site `capability` syntax from usage-site `cap X` typing,
defines runtime-provided capability injection at workflow entry and at other runtime-defined
boundaries only when they explicitly specify the same authorization contract, and keeps capability
use in Ash's explicit effect forms such as `observe Args 0`.

Follow-up cleanup of downstream planning documents that still show method-like forms remains later
work and does not change the normative specification added here.
