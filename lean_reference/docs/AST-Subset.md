# Lean AST Subset Documentation

## Overview

The Lean reference interpreter implements an **intentional subset** of the full Ash AST. This is a deliberate design choice for Phase 18 (Core ADT Operations), focusing on the essential operations needed for formal verification while avoiding complexity that would delay the core proofs.

> **Key Point**: The Lean `Expr` type is intentionally different from Rust's `Expr`. This is not a bug or temporary state—it reflects the Lean interpreter's role as a **verification-focused reference implementation** rather than a full production interpreter.

## Design Rationale

### Why a Subset?

1. **Verification Focus**: Lean's primary purpose is to serve as an executable specification for formal verification. The subset includes exactly the constructs needed for proving correctness properties.

2. **Core ADT Operations**: Phase 18 focuses on:
   - Constructor evaluation
   - Pattern matching
   - Match expressions
   - If-let expressions
   - Variable binding and lookup

3. **Avoiding Distraction**: Full arithmetic, field access, and function calls would require:
   - Additional proof complexity
   - More intricate effect tracking
   - Larger trusted computing base
   - Longer time-to-proof

4. **Differential Testing**: The subset is sufficient for meaningful differential testing against Rust while remaining tractable for verification.

## Comparison Tables

### Expr Type Comparison

| Rust Expr Variant | Lean Expr Variant | Status | Notes |
|-------------------|-------------------|--------|-------|
| `Literal(Value)` | `literal (v : Value)` | ✅ Supported | Direct correspondence |
| `Variable(Name)` | `variable (name : String)` | ✅ Supported | Direct correspondence |
| `FieldAccess { expr, field }` | — | ❌ Not supported | Use pattern matching instead |
| `IndexAccess { expr, index }` | — | ❌ Not supported | Use pattern matching on lists |
| `Unary { op, expr }` | — | ❌ Not supported | Explicit constructors preferred |
| `Binary { op, left, right }` | — | ❌ Not supported | Use match/if-let for control flow |
| `Call { func, arguments }` | — | ❌ Not supported | Inline constructor calls instead |
| — | `constructor (name : String) (fields : List (String × Expr))` | ✅ Lean-only | ADT constructor (preferred in Lean) |
| — | `tuple (elements : List Expr)` | ✅ Lean-only | Direct tuple construction |
| — | `match (scrutinee : Expr) (arms : List MatchArm)` | ✅ Lean-only | Pattern matching expression |
| — | `if_let (pattern : Pattern) (expr : Expr) (then_branch : Expr) (else_branch : Expr)` | ✅ Lean-only | Conditional pattern binding |

### Pattern Type Comparison

| Rust Pattern | Lean Pattern | Status | Notes |
|--------------|--------------|--------|-------|
| `Variable(Name)` | `variable (name : String)` | ✅ Supported | Direct correspondence |
| `Wildcard` | `wildcard` | ✅ Supported | Direct correspondence |
| `Literal(Value)` | `literal (v : Value)` | ✅ Supported | Direct correspondence |
| `Tuple(Vec<Pattern>)` | `tuple (elements : List Pattern)` | ✅ Supported | Direct correspondence |
| `Record(Vec<(Name, Pattern)>)` | `record (fields : List (String × Pattern))` | ✅ Supported | Direct correspondence |
| `List(Vec<Pattern>, Option<Name>)` | — | ⚠️ Partial | Lean has variant patterns instead |
| — | `variant (name : String) (fields : List (String × Pattern))` | ✅ Lean-only | ADT variant matching |

### Value Type Comparison

| Rust Value | Lean Value | Status | Notes |
|------------|------------|--------|-------|
| `Int(i64)` | `int (i : Int)` | ✅ Supported | Lean uses arbitrary-precision `Int` |
| `String(String)` | `string (s : String)` | ✅ Supported | Direct correspondence |
| `Bool(bool)` | `bool (b : Bool)` | ✅ Supported | Direct correspondence |
| `Null` | `null` | ✅ Supported | Direct correspondence |
| `Time(DateTime<Utc>)` | — | ❌ Not supported | Not needed for core ADT proofs |
| `Ref(String)` | — | ❌ Not supported | Use variant constructors instead |
| `List(Vec<Value>)` | `list (vs : List Value)` | ✅ Supported | Direct correspondence |
| `Record(HashMap<String, Value>)` | `record (fields : List (String × Value))` | ✅ Supported | Lean uses association list |
| `Cap(String)` | — | ❌ Not supported | Runtime capability reference |
| — | `variant (type_name : String) (variant_name : String) (fields : List (String × Value))` | ✅ Lean-only | ADT variant value |
| — | `tuple (elements : List Value)` | ✅ Lean-only | Direct tuple value |

## Workflow Constructs

The Lean interpreter does **not** implement the full `Workflow` AST from Rust. Instead, it focuses on expression evaluation:

| Rust Workflow | Lean Equivalent | Notes |
|---------------|-----------------|-------|
| `Observe { ... }` | — | Runtime capability operation |
| `Orient { ... }` | Expression evaluation | Direct expression evaluation |
| `Propose { ... }` | — | Action proposal |
| `Decide { ... }` | — | Policy decision |
| `Check { ... }` | — | Obligation checking |
| `Act { ... }` | — | Runtime action execution |
| `Let { pattern, expr, ... }` | Pattern matching | Use match/if-let expressions |
| `If { condition, ... }` | — | Use if-let with literal patterns |
| `Match` (workflow-level) | `Expr.match` | Expression-level match |
| `ForEach { ... }` | — | Iteration construct |
| `Ret { expr }` | Expression evaluation | Direct evaluation |

## Expressiveness Examples

### Rust Code (Full AST)

```rust
// Field access - NOT in Lean
let x = record.field;

// Index access - NOT in Lean
let y = list[0];

// Binary operation - NOT in Lean
let z = a + b;

// Function call - NOT in Lean
let result = compute(x, y);

// Workflow-level if
If {
    condition: Expr::Binary { ... },
    then_branch: Box::new(Workflow::Done),
    else_branch: Box::new(Workflow::Done),
}
```

### Lean Equivalent (Subset)

```lean
-- Pattern matching instead of field access
match record with
| { field := x, .. } => ...

-- Pattern matching instead of index access
match list with
| y :: _ => ...

-- Explicit constructor instead of binary op
let z := MySum { left := a, right := b }

-- Inline constructor instead of function call
let result := ComputeResult { arg1 := x, arg2 := y }

-- If-let expression instead of workflow if
if_let true = condition then_branch else_branch
```

## Future Extension Path

The subset is designed to be extensible. Future phases may add:

### Phase 19+: Expression Extensions
- `Binary` expressions for arithmetic (with effect tracking)
- `Unary` expressions for negation/logical not
- `Call` for pure functions (no side effects)

### Phase 20+: Workflow Integration
- `Observe` with capability tracking
- `Act` with provenance recording
- `Check` for obligation verification

### Phase 21+: Full Convergence
- Effect system integration
- Capability tracking
- Provenance recording

## Implications for Differential Testing

When performing differential testing between Lean and Rust:

1. **Generate Subset-Compatible Tests**: Test cases must only use constructs present in both implementations.

2. **Rust→Lean Translation**: Rust expressions using unsupported constructs must be desugared or excluded.

3. **Equivalence Boundary**: The equivalence boundary is the Lean subset. Rust may produce additional results for extended constructs, but for the subset, they must match.

4. **Test Corpus Design**: 
   - ✅ Valid: Literals, variables, constructors, tuples, match, if-let
   - ❌ Invalid: Field access, index access, unary/binary ops, function calls

## Summary

| Aspect | Rust | Lean |
|--------|------|------|
| **Purpose** | Production interpreter | Verification reference |
| **Completeness** | Full Ash language | Core ADT operations |
| **Expr variants** | 8 | 6 (different set) |
| **Pattern variants** | 5 | 6 (includes variants) |
| **Value variants** | 9 | 8 (includes tuples, variants) |
| **Workflow support** | Complete | Expression-level only |

The subset relationship is **intentional and documented**. It enables rapid verification of core ADT properties while providing a foundation for future extension.
