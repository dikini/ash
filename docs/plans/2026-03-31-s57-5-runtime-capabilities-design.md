# TASK-S57-5 Runtime Capability Syntax Design

## Context

TASK-S57-5 closes the gap between Ash capability definitions in SPEC-017 and the entry-point planning work that needs capability-typed workflow parameters such as `args: cap Args`.

## Decision

- `cap <Identifier>` is the normative usage-site type syntax for workflow parameters that receive a capability value.
- `cap` is a type-forming keyword at parameter/type positions, not a second capability declaration syntax.
- Capability declarations in source and stdlib continue to use the existing `capability` declaration form.
- Runtime-provided capabilities are injected by the runtime at workflow entry, or at another runtime-defined boundary only when that boundary explicitly specifies the same authorization contract.
- Capability invocation remains effect-first and explicit. For a read-like capability such as `Args`, the canonical source form is `observe Args 0`; dotted method-style forms are not normative.

## Rationale

This preserves Ash's capability model, where effects such as `observe`, `receive`, `set`, and `send` are the semantic center of capability use. It avoids introducing object-like method dispatch or new declaration forms that would blur the distinction between capability definition, capability typing, and capability invocation.

## Scope

This design updates:

- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- `docs/plan/tasks/TASK-S57-5-spec-017-runtime-capabilities.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

It does not implement runtime injection in Rust or stdlib modules; those remain follow-on tasks such as TASK-361 and TASK-362.
