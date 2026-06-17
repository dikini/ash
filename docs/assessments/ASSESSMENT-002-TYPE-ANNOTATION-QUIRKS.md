# Analysis: Type Annotation Quirks in fn Expressions with Imported Types

## Date: 2026-06-17
## Context: Phase 151 blocker analysis for TASK-1511
## Scope: Imported type unification in `fn` annotations, smart constructors, type leakage prevention

---

## The Core Problem

The Ash typechecker has a **fundamental limitation**: **imported types cannot be used in local type definitions**. This creates a cascade of issues:

### Issue 1: Type Definitions Cannot Reference Imported Types

```ash
// context.ash — defines GenContext
pub type GenContext = GenContext { seed: Int, size: Int };

// strategy.ash — tries to use GenContext
use test::quickcheck::context::{GenContext};

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,  -- ERROR: "Unbound variable: GenContext"
    shrink: (T) -> List<T>,
};
```

**Workaround:** `strategy.ash` defines its own `GenContext`:

```ash
// strategy.ash — workaround: duplicate the type
pub type GenContext = GenContext { seed: Int, size: Int };

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,  -- Works, but it's a DIFFERENT GenContext
    shrink: (T) -> List<T>,
};
```

This creates **two incompatible `GenContext` types** with the same structure but different module identities.

### Issue 2: fn Return Type Annotations with Imported Types

```ash
// int.ash
use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub fn ints() -> Strategy<Int> {
    Strategy { gen: gen, shrink: shrink }
}
```

The `Strategy<Int>` return type annotation uses `Strategy` which was imported. If `Strategy` has field types that reference `GenContext` (from a different module), type unification may fail or produce unexpected results.

### Issue 3: Cross-Module Type Identity

```ash
// Module A
pub type Point = Point { x: Int, y: Int };

// Module B
use a::{Point};

pub fn make_point(x: Int, y: Int) -> Point {
    Point { x: x, y: y }  -- Is this a::Point or b::Point?
}
```

The typechecker must resolve whether `Point` in the return annotation refers to the imported `a::Point` or a locally defined `b::Point`.

---

## Current State: What Works and What Doesn't

### What Works

| Scenario | Status | Example |
|----------|--------|---------|
| Using imported types in `fn` parameter types | ✅ Works | `fn foo(ctx: GenContext) -> Int` |
| Using imported types in `fn` return types | ✅ Works | `fn foo() -> Strategy<Int>` |
| Using imported types in expressions | ✅ Works | `let s = strategy::ints()` |
| Using imported types in `pub use` re-exports | ✅ Works | `pub use context::{GenContext}` |

### What Doesn't Work

| Scenario | Status | Error |
|----------|--------|-------|
| Using imported types in **local type definitions** | ❌ Broken | "Unbound variable: GenContext" |
| Using imported types in **record field types** | ❌ Broken | "Unbound variable: GenContext" |
| Cross-module type unification with duplicated types | ⚠️ Quirky | Types with same structure but different module identities don't unify |

---

## The Smart Constructor Problem

Smart constructors are functions that act as public constructors while keeping the type opaque (private). This is a common pattern in functional programming:

```ash
// Internal module: types.ash
pub type Message = Message {
    role: Role,
    content: String,
    tool_calls: List<ToolCall>,
};

-- The constructor is NOT exported (no pub)
-- Users cannot construct Message directly

// Public module: prompt.ash
use types::{Message};

-- Smart constructor: public function that creates Message
pub fn user_message(content: String) -> Message {
    Message { role: User, content: content, tool_calls: [] }
}
```

### The Quirk: Type Leakage in Definitions

Even if the type constructor is private, the **type name** appears in public function signatures:

```ash
pub fn user_message(content: String) -> Message {
    -- Message type is visible in the signature
    -- But the constructor is private
}
```

This is fine — the type is public, only the constructor is private. But the issue arises when:

1. **The type is not imported** but values of that type participate in type checking
2. **The type appears in definitions** without being explicitly imported
3. **Type inference** produces types that are not imported

### Example of the Subtle Bug

```ash
// Module A: internal.ash
pub type Secret = Secret { value: Int };  -- pub type, private constructor

fn make_secret(v: Int) -> Secret {        -- private smart constructor
    Secret { value: v }
}

// Module B: public.ash
-- Does NOT import Secret

pub fn double_secret(s) -> Secret {  -- Type inferred as Secret, but Secret not imported!
    Secret { value: s.value * 2 }     -- ERROR: Secret not in scope
}
```

The problem: `double_secret` takes a parameter `s` with no type annotation. The typechecker infers `s: Secret` from the body. But `Secret` is not imported in `public.ash`. This causes:

1. **Type inference succeeds** (Secret is known from the expression)
2. **Type checking fails** (Secret is not in scope for the return type)
3. **Or worse:** Type checking succeeds but produces a type that leaks the internal module's type

---

## Root Cause Analysis

### Cause 1: Type Environment Scope

The type environment (`TypeEnv`) is populated in this order:
1. Parse module declarations
2. Register local types
3. Process imports
4. Typecheck expressions

But **type definitions are processed before imports are resolved**, so imported types are not available when defining local types.

### Cause 2: Type Name Resolution

Type names in annotations are resolved by looking up the name in the current scope. If the name is not imported, it's not found. But if the type is inferred from an expression, the type's internal representation (type ID) is used, which may not have a name in the current scope.

### Cause 3: Cross-Module Type Identity

Types are identified by their **module + name**. Two types with the same structure but different module origins are different types. This is correct for nominal typing, but it means:

- `context::GenContext` ≠ `strategy::GenContext` (even if structurally identical)
- Type unification fails when mixing these types

---

## Solutions and Trade-offs

### Solution 1: Two-Pass Type Processing (Recommended)

**Change the parser/typechecker to process imports before type definitions.**

```
Current order:
1. Parse all declarations
2. Register local types
3. Process imports
4. Typecheck expressions

New order:
1. Parse all declarations
2. Process imports (register imported types in TypeEnv)
3. Register local types (can now reference imported types)
4. Typecheck expressions
```

**Pros:**
- Solves the core issue: imported types available in type definitions
- Enables proper cross-module type usage
- No workaround needed for `GenContext` duplication

**Cons:**
- Requires parser/typechecker pipeline changes
- May affect performance (two-pass processing)
- Risk of breaking existing code

**Effort:** Medium (2-3 days)

### Solution 2: Type Forwarding/Aliasing

**Allow types to be explicitly forwarded/aliased across modules.**

```ash
// strategy.ash
pub type GenContext = context::GenContext;  -- Forward the type

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,  -- Now works: GenContext is local alias
    shrink: (T) -> List<T>,
};
```

**Pros:**
- Explicit control over type visibility
- No parser changes needed
- Works with current pipeline

**Cons:**
- Requires new syntax or semantics
- May confuse users (is it a new type or an alias?)
- Doesn't solve the inference leakage problem

**Effort:** Medium (2-3 days)

### Solution 3: Implicit Type Import from Expressions

**When type inference produces a type not in scope, automatically import it or report a clear error.**

```ash
// public.ash
-- Does NOT import Secret

pub fn double_secret(s) -> ??? {  -- Type inferred as Secret
    Secret { value: s.value * 2 }  -- ERROR: Secret must be imported to use in signature
}
```

The typechecker would report:
```
Error: Type `Secret` (from module `internal`) is used in the signature
of `double_secret` but is not imported. Add:

    use internal::{Secret};

Or use an explicit type annotation with a local type.
```

**Pros:**
- Clear error messages
- Prevents accidental type leakage
- Guides users to correct code

**Cons:**
- Doesn't solve the core issue
- Just better error reporting
- Still requires manual imports

**Effort:** Low (1 day)

### Solution 4: Opaque Type Handles

**Distinguish between "type name" and "type identity" in the type system.**

```ash
// internal.ash
pub type Secret = Secret { value: Int };  -- pub type name, opaque identity

// public.ash
use internal::{Secret};  -- Imports the type NAME, not the constructor

pub fn double_secret(s: Secret) -> Secret {
    -- Can use Secret in signatures
    -- Cannot construct Secret (constructor not imported)
}
```

**Pros:**
- Clean separation of type names and constructors
- Enables smart constructors naturally
- Prevents constructor leakage

**Cons:**
- Complex type system change
- Requires new syntax/semantics
- May confuse users

**Effort:** High (5-7 days)

---

## Recommended Approach: Combined Solution

### Phase 1: Two-Pass Processing (TASK-1540)

Implement two-pass type processing to allow imported types in type definitions.

```
Pass 1: Collect all imports and register imported types in TypeEnv
Pass 2: Process local type definitions (can now reference imported types)
Pass 3: Typecheck expressions
```

This solves the `GenContext` duplication issue and enables proper cross-module type usage.

### Phase 2: Better Error Messages (TASK-1541)

When type inference produces a type not in scope, report a clear error with the module path and import suggestion.

### Phase 3: Constructor Visibility (TASK-1542)

Ensure that `pub type` exports the type name but not the constructor, while `pub type` with `pub` fields exports both. This enables smart constructors naturally.

```ash
pub type Secret = Secret { value: Int };  -- Type public, constructor private
pub type Public = Public { pub value: Int };  -- Type and constructor public
```

---

## Impact on TASK-1511

### Current Blockers for TASK-1511

| Blocker | Status | Solution |
|---------|--------|----------|
| `let` destructors for records | 📝 Phase 152 | TASK-1520-TASK-1522 |
| Imported type unification in `fn` annotations | 📝 This analysis | TASK-1540-TASK-1542 |
| List concatenation / indexing | 📝 Phase 153 | TASK-1530-TASK-1532 |
| Closures with variable capture | 📝 Phase 152 | TASK-1520-TASK-1524 |

### What TASK-1511 Needs

The QuickCheck combinators need to:
1. Define types that reference imported types (e.g., `Strategy<T>` using `GenContext`)
2. Use imported types in `fn` return annotations (e.g., `fn ints() -> Strategy<Int>`)
3. Use smart constructors to create values without exposing internal types

All of these are blocked by the type annotation quirks.

---

## Verification Strategy

### Test Case 1: Imported Type in Type Definition

```ash
// module_a.ash
pub type Point = Point { x: Int, y: Int };

// module_b.ash
use module_a::{Point};

pub type Line = Line { start: Point, end: Point };  -- Should work
```

### Test Case 2: Imported Type in fn Return Annotation

```ash
// module_a.ash
pub type Point = Point { x: Int, y: Int };

// module_b.ash
use module_a::{Point};

pub fn origin() -> Point {
    Point { x: 0, y: 0 }  -- Should work
}
```

### Test Case 3: Smart Constructor

```ash
// internal.ash
pub type Secret = Secret { value: Int };  -- pub type, private constructor

fn make_secret(v: Int) -> Secret {
    Secret { value: v }
}

// public.ash
use internal::{Secret};

pub fn double(s: Secret) -> Secret {
    -- Can use Secret in signature
    -- Cannot construct Secret (constructor private)
}
```

### Test Case 4: Type Inference Leakage Prevention

```ash
// internal.ash
pub type Secret = Secret { value: Int };

// public.ash
-- Does NOT import Secret

pub fn bad(s) -> Secret {  -- Should error: Secret not imported
    Secret { value: s.value * 2 }
}
```

---

## Conclusion

The "type annotation quirks" are a **fundamental type system limitation** where:
1. Imported types cannot be used in local type definitions
2. Type inference may produce types not in scope
3. Cross-module type identity is strictly nominal (module + name)

The **recommended solution** is a **combined approach**:
1. **Two-pass type processing** (TASK-1540) to allow imported types in definitions
2. **Better error messages** (TASK-1541) for type inference leakage
3. **Constructor visibility** (TASK-1542) for smart constructors

This unblocks TASK-1511 and enables proper modular type design in Ash.
