# TASK-1558: Cookbook Destructor Patterns

## Status: ✅ Complete

## Description

Add destructor examples to the cookbook showing common patterns.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)

## Content to Add

- Extracting fields from a record
- Swapping variables via tuple destructuring
- Working with Strategy values (QuickCheck)
- Working with GenContext values
- Partial extraction (ignoring unneeded fields)
- Renaming fields for clarity
- Common error patterns and how to fix them

## Examples

```ash
-- Extracting fields
let { gen, shrink } = strategy;

-- Swapping variables
let (a, b) = (b, a);

-- Partial extraction
let { gen } = strategy;  -- Only need gen

-- Renaming
let { gen: generator } = strategy;

-- With QuickCheck combinators
pub fn map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B> {
    let { gen, shrink } = s;
    Strategy {
        gen: fn(ctx) { f(gen(ctx)) },
        shrink: fn(b) { [] }
    }
}
```

## Verification

- Documentation renders correctly
- Examples are accurate and tested
- Cross-references to other docs work
