# TASK-1607: Write operational semantics for new term forms

## Status: 📝 Planned

## Description

Write a new operational semantics document covering the expanded CPS IR forms: `Value::Record`, `Value::Tuple`, `Atom::ConstructorName`, `PrimOp::RecordGet`/`TupleGet`, and `Term::Match`. This is a **new document** that extends SPEC-099b without modifying it.

## Specification Reference

- [SPEC-099b: Target Operational Semantics](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — Base document (§1-§7)
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1600: Record/Tuple values
- ✅ TASK-1601: Field access primitives
- ✅ TASK-1602: Constructor tags
- ✅ TASK-1603: Match dispatch

## Requirements

### Functional Requirements

1. Create new document: `docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md`
2. Document syntax extensions (§1 additions)
3. Document evaluation rules for new forms (§2-§3 additions)
4. Document mutual recursion desugaring pattern (§4)
5. Cross-reference SPEC-099b for all unchanged rules

### Document Structure

```markdown
# SPEC-099c: CPS IR Expanded Operational Semantics

**Status:** Draft — extends SPEC-099b with structured data and pattern matching
**Scope:** Operational semantics for the expanded CPS IR implemented in Phase 160
**Depends on:** SPEC-099b (Base Operational Semantics), SPEC-098b (Target IR)

## §1 Syntax Extensions (to SPEC-099b §1)

### §1.2 Values (extended)

```text
v ::= ... (from SPEC-099b §1.2)
    | Record { fields: [(x, a), ...] }
    | Tuple { elems: [a, ...] }
```

### §1.3 Atoms (extended)

```text
a ::= ... (from SPEC-099b §1.1)
    | ConstructorName(n)
```

### §1.4 Terms (extended)

```text
t ::= ... (from SPEC-099b §1.3)
    | Match { scrutinee: a, arms: [(n, t), ...], default: t? }
```

## §2 Record and Tuple Rules

### §2.1 Record Construction

```text
eval(aᵢ, η) = aᵢ' for each field (xᵢ, aᵢ)
-----------------------------------
⟨Record { fields: [(x₁, a₁), ...] }, η⟩ ⇓ Record { fields: [(x₁, a₁'), ...] }
```

### §2.2 Tuple Construction

```text
eval(aᵢ, η) = aᵢ' for each element
-----------------------------------
⟨Tuple { elems: [a₁, ...] }, η⟩ ⇓ Tuple { elems: [a₁', ...] }
```

### §2.3 RecordGet

```text
eval(a, η) = a'
lookup(a', η) = Record { fields: [... (x, v) ...] }
-----------------------------------
⟨RecordGet(x, a), η, χ⟩ ⇓ v
```

### §2.4 TupleGet

```text
eval(a, η) = a'
lookup(a', η) = Tuple { elems: [...] }
elems[i] = v
-----------------------------------
⟨TupleGet(i, a), η, χ⟩ ⇓ v
```

## §3 Match Dispatch Rules

### §3.1 Match (matching arm)

```text
eval(a, η) = a'
lookup(a', η) = Tuple { elems: [ConstructorName(n), ...] }
arms contains (n, t)
-----------------------------------
⟨Match { scrutinee: a, arms: [... (n, t) ...], ... }, η, χ⟩ ⇓ ⟨t, η, χ⟩
```

### §3.2 Match (default)

```text
eval(a, η) = a'
lookup(a', η) = Tuple { elems: [ConstructorName(n), ...] }
no arm matches n
default = Some(t)
-----------------------------------
⟨Match { ... }, η, χ⟩ ⇓ ⟨t, η, χ⟩
```

### §3.3 Match (no match, no default)

```text
eval(a, η) = a'
lookup(a', η) = Tuple { elems: [ConstructorName(n), ...] }
no arm matches n
default = None
-----------------------------------
⟨Match { ... }, η, χ⟩ ⇓ Stuck(MatchError(n))
```

## §4 Mutual Recursion Desugaring

### §4.1 Pattern

Mutual recursion is desugared to single `LetRec` with a tuple of lambdas:

```text
letrec even odd = ...
  even calls odd
  odd calls even

-- desugars to:

letrec pair = (tuple even odd) in ...
  where even = (lam [n] k ... (tuple_get 1 pair) ...)
        odd  = (lam [n] k ... (tuple_get 0 pair) ...)
```

### §4.2 Semantics

The placeholder/backfill mechanism (SPEC-099b §5.1) handles this correctly because:
1. `pair` is bound to `Null` as placeholder
2. The tuple is constructed with lambdas that capture `pair` by `Var` reference
3. When `pair` is backfilled with the actual tuple, the lambdas see the updated value
4. `tuple_get` resolves to the correct lambda through environment lookup

## §5 See Also

- [SPEC-099b: Base Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md)
- [PLAN-160: CPS IR Runtime Expansion](../plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 -c "import os; assert os.path.exists('docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md')"
  - git diff --check
checklist:
  - [ ] Document exists and is readable
  - [ ] All new forms have inference rules
  - [ ] Mutual recursion desugaring is documented
  - [ ] Cross-references to SPEC-099b are correct
  - [ ] No modification to SPEC-099b
```

## Dependencies for Next Task

- Provides formal semantics for TASK-1608 (reference docs)

## Notes

- This is a **new document**, not a modification to SPEC-099b. SPEC-099b remains frozen as the Phase 159 baseline.
- The document should be precise enough that a future Lean 4 mechanization could use it.
- Keep rules in the same style as SPEC-099b (big-step, environment + handler chain).
