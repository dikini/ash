# TASK-S57-5: Update SPEC-017 with Runtime-Provided Capability Syntax

## Status: ⬜ Pending

## Description

Update SPEC-017 (Capability Integration) with normative syntax for runtime-provided/injected capabilities, resolving the mismatch between `cap Args` usage in 57B and current `capability` syntax in SPEC-017.

## Background

Per architectural review, 57B would like to use:
```ash
workflow main(args: cap Args) -> Result<(), RuntimeError> {
  args.get(0)
}
```

**Current SPEC-017 status:** The specification for runtime-provided/injected capabilities is not fully defined. This task establishes the normative syntax and semantics.

**Note:** Current SPEC-017-CAPABILITY-INTEGRATION.md:8-32 focuses on capability definitions, not runtime-injected capability types. This task extends SPEC-017 for the entry point use case.

**Questions:**
- Is `cap Args` a type annotation? A keyword?
- How is `Args` defined as a capability type?
- What's the relationship between `capability` declarations and `cap` types?

## Requirements

Update SPEC-017 with:

1. **Capability type syntax**: How to write `cap Args` as parameter type
2. **Runtime-provided capabilities**: Special handling for injected capabilities
3. **Capability definition**: How stdlib defines `Args` as capability
4. **Usage syntax**: How workflows use capabilities (`act` vs method calls)

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

How does stdlib declare `Args` as a capability?

Option 1:
```ash
-- std/src/runtime/args.ash
capability Args {
  fn get(index: Int) -> Option<String>
  fn all() -> [String]
}
```

Option 2:
```ash
pub cap Args = capability {  -- type alias style
  fn get(index: Int) -> Option<String>
}
```

## Open Questions

### Q1: Type vs Keyword
- Is `cap` a keyword introducing a capability type?
- Or is `cap Args` shorthand for `Capability<Args>`?

### Q2: Method Call Syntax
- How do you call capability methods?
- `args.get(0)`? `act args.get(0)`? `act args with Get(0)`?

### Q3: Effect Tracking
- Do capability calls have effects?
- How are they tracked in type system?

### Q4: Multiple Capabilities
- Can a workflow have multiple capability params?
- `main(args: cap Args, stdout: cap Stdout)`?

## Acceptance Criteria

- [ ] Capability type syntax (`cap X`) normatively defined
- [ ] Runtime-injected capability mechanism specified
- [ ] Capability definition syntax for stdlib
- [ ] Usage/call syntax for capability methods
- [ ] Examples show valid capability usage
- [ ] 57B tasks can use normative syntax

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
