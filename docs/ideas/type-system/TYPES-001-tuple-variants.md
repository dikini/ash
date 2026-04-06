---
status: candidate
created: 2026-03-30
last-revised: 2026-04-06
related-plan-tasks: [TASK-413]
tags: [type-system, syntax, variants, tuples, adt]
---

# TYPES-001: Canonical Tuple Variant Syntax for ADTs

## Problem Statement

The current Ash ADT specs define enum variants in record style:

```ash
type Result<T, E> = Ok { value: T } | Err { error: E };
```

That works well for self-describing payloads, but it is awkward for newtype-like and tuple-like
constructors such as `RuntimeError`, wrapper types, and compact sum-type payloads.

Ash therefore needs one canonical source-level syntax for tuple variants.

## Decision

Ash should use explicit parenthesized tuple payload syntax for tuple variants.

Canonical source form:

```ash
type RuntimeError = RuntimeError(Int, String);
type Box<T> = Box(T);
type Status = Pending | Processing | Completed;
```

This replaces the earlier option inventory. Tuple variants are no longer an open syntax question in
this exploration.

## Why This Syntax

This choice is the best fit for Ash because it is:

1. unambiguous;
2. visually distinct from record variants;
3. consistent with tuple type syntax;
4. straightforward to lower into explicit constructor metadata;
5. easier to explain in constructor expressions and patterns than space-separated payload syntax.

The rejected alternative:

```ash
type RuntimeError = RuntimeError Int String;
```

looks concise but creates unnecessary parsing and readability ambiguity around constructor/type
application boundaries.

## Canonical Source Model

Ash should support three variant payload shapes at the source level:

- unit variants
- record variants
- tuple variants

Canonical examples:

```ash
type Status = Pending | Processing | Completed;

type Option<T> =
  Some { value: T }
| None;

type RuntimeError = RuntimeError(Int, String);

type PairBox<T, U> = PairBox(T, U);
```

### Canonical Reading

- `Pending` is a unit variant
- `Some { value: T }` is a record variant
- `RuntimeError(Int, String)` is a tuple variant

## Constructor Expressions

Tuple variants should use the same parenthesized payload form in expressions:

```ash
let err = RuntimeError(2, "missing config");
let wrapped = Box(value);
```

Record variants remain record-shaped:

```ash
let ok = Ok { value: 42 };
```

Unit variants remain bare constructor names:

```ash
let status = Pending;
```

## Pattern Syntax

Tuple variants should be matched positionally:

```ash
match err {
  RuntimeError(code, msg) => msg,
}
```

Nested examples:

```ash
match result {
  Ok { value: RuntimeError(code, msg) } => msg,
  Err { error: e } => e,
}
```

Record variants remain field-based:

```ash
match opt {
  Some { value: x } => x,
  None => 0,
}
```

## MVP Scope

The MVP source contract should include:

1. tuple-variant declarations in type definitions;
2. tuple-variant constructor expressions;
3. tuple-variant patterns in `match` and `if let`-style lowering contexts.

The MVP should explicitly exclude positional field projection syntax such as:

```ash
err.0
err.1
```

For the first pass, tuple-variant payload extraction should happen through pattern matching rather
than positional projection.

## Source-Level Grammar Direction

Canonical grammar shape:

```bnf
variant         ::= IDENTIFIER variant_payload?
variant_payload ::= record_payload | tuple_payload
record_payload  ::= "{" field_list "}"
tuple_payload   ::= "(" type_list ")"
type_list       ::= type ("," type)*
```

Constructor/pattern surface should mirror that payload shape:

```bnf
constructor_expr    ::= IDENTIFIER tuple_expr_payload?
tuple_expr_payload  ::= "(" expr_list ")"
variant_pattern     ::= IDENTIFIER tuple_pattern_payload?
tuple_pattern_payload ::= "(" pattern_list ")"
```

This note intentionally fixes the tuple-variant surface shape. It does not require the runtime to
store tuple payloads as a distinct user-visible value form. Internal elaboration remains an
implementation concern so long as constructor typing and pattern matching preserve the positional
source contract.

## Spec Impact

The main normative follow-on should be:

- `SPEC-020` for ADT source definitions, constructor expressions, and variant patterns;
- `SPEC-002` for parser-facing grammar;
- `SPEC-003` and the type/runtime boundary references for tuple-variant constructor/pattern typing;
- `SPEC-004` only insofar as runtime/pattern semantics need to acknowledge tuple-variant source
  payloads.

## Design Constraints

This decision carries a few important constraints:

1. record and tuple variants must stay distinct source forms;
2. pattern syntax must preserve the same distinction;
3. no implicit field names should be exposed at source level for tuple variants;
4. internal implementation may elaborate tuple payloads however it likes, but that elaboration must
   not become the user-facing contract.

## Non-Goals

This note does not define:

- positional field projection syntax;
- automatic conversion between tuple and record variants;
- tuple values as a separate first-class runtime value family;
- deriving or generic-programming behavior for tuple variants.

## Recommended Follow-On

Promote this note through a narrow contract-first task:

- [TASK-413: Canonical Tuple Variant Syntax and ADT Contract Alignment](../../plan/tasks/TASK-413-canonical-tuple-variant-syntax.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration opened | Needed a place to evaluate tuple-variant syntax for `RuntimeError` and wrapper types |
| 2026-04-06 | Explicit parenthesized tuple payload syntax chosen as canonical | Lowest ambiguity, cleanest parser story, best fit for constructor/pattern symmetry |
