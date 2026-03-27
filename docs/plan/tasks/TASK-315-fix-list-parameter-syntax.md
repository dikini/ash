# TASK-315: Fix Parser List<T> Syntax in Workflow Parameters

## Status: 📝 Planned

## Problem

The parser does not accept `List<Int>` generic syntax in workflow parameter declarations.

**Expected:** `workflow sum(items: List<Int>) { ... }` should parse

**Current:** Parse error on `List<Int>`

## Reproduction

```bash
$ echo 'workflow sum(items: List<Int>) { ret 42; }' > test.ash
$ ash run test.ash
error: parse error: Parsing Error: ...
```

## Root Cause

The grammar for `value_exposure` (parameter type annotations) likely only accepts simple IDENTIFIER tokens, not generic type constructors like `List<Int>`.

SPEC-002 grammar allows generics, but the parser implementation for parameter types may be incomplete.

## Files to Investigate

- `crates/ash-parser/src/` - Type parsing, specifically for parameter declarations
- `crates/ash-parser/src/grammar.rs` or similar
- Look for `value_exposure` parsing

## Implementation

Extend the parser to accept generic type syntax (TypeName<T>) in parameter position.

Current (likely):
```rust
// value_exposure := IDENTIFIER*
```

Needed:
```rust
// value_exposure := (IDENTIFIER | IDENTIFIER "<" type_list ">")*
```

## Verification

After fix:
```bash
$ ash run test.ash
42  # Success
```

And `test_list_workflow_parameter` test should pass.

## Spec Impact

SPEC-002 (Parser) may need clarification on what types are allowed in `value_exposure`.

## Completion Checklist

- [ ] `List<Int>` parses in workflow parameters
- [ ] `List<String>` parses
- [ ] Other generic types parse
- [ ] `test_list_workflow_parameter` test passes
- [ ] All other tests still pass
- [ ] CHANGELOG.md updated
- [ ] SPEC-002 updated with parameter type grammar

**Estimated Hours:** 8
**Priority:** Medium (feature completion)
