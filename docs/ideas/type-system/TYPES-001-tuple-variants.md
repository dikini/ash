---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [type-system, syntax, variants, tuples, adt]
---

# TYPES-001: Tuple Variant Syntax for ADTs

## Problem Statement

The current Ash spec (SPEC-002, SPEC-020) only defines **record-style** enum variants:

```ash
type Result<T, E> = Ok { value: T } | Err { error: E };
```

But for `RuntimeError` and similar types, we want **tuple-style** variants:

```ash
type RuntimeError = RuntimeError Int String;  -- exit_code, message
```

This exploration defines the syntax and semantics for tuple variants.

## Current Grammar (SPEC-002)

```
variant         ::= IDENTIFIER ("{" field_list "}")?
field_list      ::= field ("," field)*
field           ::= IDENTIFIER ":" type
```

This only supports record-style variants with named fields.

## Proposed Extensions

### Option A: Multiple Type Arguments (Space-Separated)

```
variant         ::= IDENTIFIER ("{" field_list "}" | type*)?
```

Examples:
```ash
type RuntimeError = RuntimeError Int String;     -- two anonymous fields
type Box<T> = Box T;                              -- one field (newtype)
type Status = Pending | Processing | Completed;  -- zero fields (unit)
```

**Concern:** `IDENTIFIER type*` is ambiguous. `Foo Bar Baz` could be:
- Variant `Foo` with two fields `Bar` and `Baz`
- Generic application `Foo<Bar>` with stray `Baz`

### Option B: Explicit Tuple Syntax (Parentheses)

```
variant         ::= IDENTIFIER ("{" field_list "}" | "(" type_list ")")?
type_list       ::= type ("," type)*
```

Examples:
```ash
type RuntimeError = RuntimeError (Int, String);  -- tuple payload
type Box<T> = Box (T);                            -- single-element tuple
type Status = Pending | Processing | Completed;   -- no parens = unit
```

**Benefits:**
- Unambiguous
- Consistent with tuple type syntax `(Int, String)`
- Clear distinction between record `{ }` and tuple `( )` variants

### Option C: Hybrid - Allow Both in Different Contexts

- Record syntax for most variants (clear field names)
- Tuple syntax for simple wrappers (`RuntimeError`, `Box`)

## Use Cases

| Type | Preferred Syntax | Rationale |
|------|------------------|-----------|
| `RuntimeError` | `RuntimeError (Int, String)` | Simple wrapper, no semantic field names needed |
| `Result<T, E>` | `Ok { value: T }` | Clear semantics, self-documenting |
| `Option<T>` | `Some { value: T }` / `None` | Record for `Some`, unit for `None` |
| `Box<T>` | `Box (T)` | Newtype pattern, no name needed |

## Open Questions

1. **Should we support both record and tuple syntax?** Or mandate one style?
2. **Access syntax:** If `err: RuntimeError`, how do we extract fields?
   - `err.0`, `err.1` (positional)?
   - `err.exit_code`, `err.message` (requires named fields)?
   - Pattern matching only?
3. **Pattern matching:** How do we match tuple variants?
   ```ash
   match err {
     RuntimeError (code, msg) => ...
   }
   ```
4. **Generic tuple variants:** Can type parameters appear in tuple position?
   ```ash
   type Pair<T, U> = Pair (T, U);
   ```

## Related

- MCE-001 (Entry Point): Uses `RuntimeError` as motivating example
- SPEC-020 (ADT Types): Current canonical ADT specification
- SPEC-002 (Surface): Grammar for type definitions

## Decision Needed

Which syntax option should be canonical for Ash?

| Option | Status | Notes |
|--------|--------|-------|
| A: Space-separated | Draft | Ambiguity concerns |
| B: Explicit tuple `()` | Draft | Recommended |
| C: Hybrid | Draft | Most flexible |
