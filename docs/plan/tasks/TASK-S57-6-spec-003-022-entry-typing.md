# TASK-S57-6: Update SPEC-003/SPEC-022 with Entry Workflow Typing Contract

## Status: ⬜ Pending

## Description

Update SPEC-003 (Type System) and/or SPEC-022 (Workflow Typing) with the normative typing contract for entry workflows: `main` workflow with signature `Result<(), RuntimeError>` and capability-only parameters.

## Background

Per architectural review, TASK-364 assumes:
- Entry workflow named `main`
- Return type exactly `Result<(), RuntimeError>`
- Only capability parameters

This typing contract is plausible but not yet grounded in SPEC. Needs explicit specification before implementation.

## Requirements

Update SPEC-003 or SPEC-022 with:

1. **Entry workflow identifier**: Named `main`
2. **Return type constraint**: Must be `Result<(), RuntimeError>`
3. **Parameter constraints**: Only capability types allowed
4. **Typechecking rule**: How entry workflow type is verified

## SPEC Sections to Update

### Option A: Update SPEC-022 (Workflow Typing)

Add section "Entry Workflow Typing":

```
Entry Workflow Judgment:

Γ ⊢ entry_main : () -> Result<(), RuntimeError> ▷ ε
---------------------------------------------------
Γ ⊢wf main valid_entry

Constraints:
- Name: "main"
- Parameters: All capability types (cap X)
- Return: Result<(), RuntimeError>
```

### Option B: Update SPEC-003 (Type System)

Define entry workflow as special type rule:

```
EntrySignature ::= {
  name: "main",
  params: [capability_type*],
  return: Result<(), RuntimeError>
}
```

### Option C: Keep in Runtime/CLI Spec

Don't put in type system; treat as runtime/CLI contract in SPEC-005.

## Open Questions

### Q1: Exact Return Type
- Is `Result<(), RuntimeError>` exact match required?
- Or is `Result<T, RuntimeError>` for any `T` acceptable?
- If `T` must be `()`, how is that enforced?

### Q2: Generic RuntimeError
- Is `RuntimeError` a concrete type or type alias?
- How is it defined in stdlib vs known to type checker?

### Q3: Parameter Count
- Can `main` have zero parameters?
- Must it have at least `cap Args`?
- Maximum parameters?

### Q4: Effect Annotation
- Does `main` declare effects?
- Or are effects inferred from body?

### Q5: Where to Specify
- Type system (SPEC-003)?
- Workflow typing (SPEC-022)?
- CLI spec (SPEC-005)?
- New "Entry Point" spec section?

## Acceptance Criteria

- [ ] Entry workflow name constraint specified
- [ ] Return type constraint specified (exactly `Result<(), RuntimeError>`)
- [ ] Parameter constraint specified (capabilities only)
- [ ] Typechecking judgment/rule defined
- [ ] Error cases specified (wrong name, wrong return, non-cap params)
- [ ] 57B TASK-364 can implement against this spec

## Related

- SPEC-003: Type system
- SPEC-022: Workflow typing
- SPEC-005: CLI (may reference this contract)
- MCE-001: Entry point design
- TASK-364: Main verification (blocked on this)
- TYPES-001: RuntimeError syntax (related)

## Est. Hours: 2-3

## Blocking

- TASK-364: Main verification (needs normative spec to verify against)
- TASK-366: CLI error messages (needs to reference spec for errors)
