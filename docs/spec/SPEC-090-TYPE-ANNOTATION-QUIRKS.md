# SPEC-090: Fix Type Annotation Quirks in fn Expressions with Imported Types

**Status:** Implemented MVP (Phase 154)
**Date:** 2026-06-17
**Amends:** [SPEC-057](SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-072](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Builds on:** [ASSESSMENT-002](../assessments/ASSESSMENT-002-TYPE-ANNOTATION-QUIRKS.md)
**Plan:** [PLAN-154](../plan/PLAN-154-TYPE-ANNOTATION-QUIRKS.md)

## 1. Summary

Fix the type system limitation where imported types cannot be used in local type definitions, `fn` return type annotations, and record field types. This unblocks modular type design, smart constructors, and cross-module type composition.

## 2. Motivation

Currently, the Ash typechecker processes local type definitions before resolving imports. This means:

```ash
// context.ash
pub type GenContext = GenContext { seed: Int, size: Int };

// strategy.ash
use test::quickcheck::context::{GenContext};

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,  -- ERROR: "Unbound variable: GenContext"
    shrink: (T) -> List<T>,
};
```

The workaround is to duplicate the type definition, creating two incompatible types with the same structure.

## 3. Core Design

### 3.1 Two-Pass Type Processing

Change the parser/typechecker pipeline from single-pass to two-pass:

```
Pass 1: Import Resolution
- Parse all `use` statements
- Register imported types in TypeEnv
- Build import map (name -> module::type)

Pass 2: Type Definition Processing
- Process local `type` definitions (can now reference imported types)
- Process `fn` signatures (can use imported types in annotations)
- Process `interface` definitions (can reference imported types in methods)

Pass 3: Expression Type Checking
- Typecheck expressions with full type environment
```

### 3.2 Type Name Resolution Rules

| Context | Resolution Order | Example |
|---------|-----------------|---------|
| Type definition | 1. Local types, 2. Imported types, 3. Builtins | `GenContext` -> `context::GenContext` |
| fn parameter | 1. Local types, 2. Imported types, 3. Builtins | `ctx: GenContext` -> `context::GenContext` |
| fn return | 1. Local types, 2. Imported types, 3. Builtins | `-> Strategy<Int>` -> `strategy::Strategy<Int>` |
| Record field | 1. Local types, 2. Imported types, 3. Builtins | `start: Point` -> `point::Point` |

### 3.3 Smart Constructor Support

Types that appear in public function signatures are **implicitly public** — their names can be used in other modules, but their constructors remain private. This enables the smart constructor pattern naturally:

```ash
// internal.ash
pub type Secret = Secret { value: Int };  -- Type public, constructor public

// OR: private type with public smart constructor
fn make_secret(v: Int) -> Secret {        -- Private type, private constructor
    Secret { value: v }
}

pub fn make_secret(v: Int) -> Secret {     -- Public function makes Secret nameable
    Secret { value: v }
}
```

When a private type appears in a `pub fn` signature, it becomes **publicly nameable** but **not constructable** from outside the module:

```ash
// public.ash
use internal::{make_secret, get_value};  -- Import functions, NOT the type

pub fn double_secret(v: Int) -> Secret {  -- ✅ Secret is public because it appears in pub fn
    make_secret(get_value(make_secret(v)) * 2)  -- Use smart constructor
}

pub fn bad() -> Secret {
    Secret { value: 42 }  -- ❌ Constructor is private (not exported)
}
```

### 3.4 Opaque Values from Smart Constructors

Values produced by smart constructors are **opaque** — they cannot be pattern matched, destructured, or have their fields accessed directly. Only valid public operations and functions are legal:

```ash
// public.ash
use internal::{make_secret, get_value, is_valid};

pub fn process(s: Secret) -> Secret {
    -- ✅ Valid: use public functions
    if is_valid(s) {
        make_secret(get_value(s) * 2)
    } else {
        make_secret(0)
    }
    
    -- ❌ Invalid: cannot pattern match on opaque value
    -- match s {
    --     Secret { value: v } => ...  -- ERROR: constructor not exported
    -- }
    
    -- ❌ Invalid: cannot destructure
    -- let Secret { value: v } = s;  -- ERROR: constructor not exported
    
    -- ❌ Invalid: cannot access fields directly
    -- s.value  -- ERROR: field access on opaque value
}
```

This ensures that the module defining the type maintains full control over how values are created and manipulated, preventing users from bypassing invariants enforced by smart constructors.

### 3.5 Type Inference Leakage Prevention

When type inference produces a type not in the current scope, the typechecker must:
1. Report a clear error with the module path
2. Suggest the import statement needed
3. Not silently leak internal types

```
Error: Type `Secret` (from module `internal`) is used in the signature
of `double_secret` but is not imported.

Help: Add the following import:
    use internal::{Secret};

Or use an explicit type annotation with a local type.
```

## 4. Acceptance Criteria

### C90-1: Imported types in type definitions

```ash
// module_a.ash
pub type Point = Point { x: Int, y: Int };

// module_b.ash
use module_a::{Point};

pub type Line = Line { start: Point, end: Point };  -- Must work
```

### C90-2: Imported types in fn return annotations

```ash
// module_a.ash
pub type Point = Point { x: Int, y: Int };

// module_b.ash
use module_a::{Point};

pub fn origin() -> Point {
    Point { x: 0, y: 0 }  -- Must work
}
```

### C90-3: Smart constructors with opaque values

```ash
// internal.ash
fn make_secret(v: Int) -> Secret {  -- Private type, private constructor
    Secret { value: v }
}

pub fn make_secret(v: Int) -> Secret {  -- Public function makes Secret nameable
    Secret { value: v }
}

pub fn get_value(s: Secret) -> Int {  -- Public accessor
    s.value
}

// public.ash
use internal::{make_secret, get_value};

pub fn double(v: Int) -> Secret {  -- ✅ Secret is public (appears in pub fn)
    make_secret(get_value(make_secret(v)) * 2)  -- Must work
}

pub fn bad() -> Secret {
    Secret { value: 42 }  -- ❌ Constructor is private (not exported)
}
```

### C90-4: Type inference leakage prevention

```ash
// internal.ash
pub type Secret = Secret { value: Int };

// public.ash
-- Does NOT import Secret

pub fn bad(s) -> Secret {  -- Must error: Secret not imported
    Secret { value: s.value * 2 }
}
```

### C90-5: Cross-module type unification

```ash
// context.ash
pub type GenContext = GenContext { seed: Int, size: Int };

// strategy.ash
use test::quickcheck::context::{GenContext};

pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,  -- Must work
    shrink: (T) -> List<T>,
};

// int.ash
use test::quickcheck::strategy::{Strategy};
use test::quickcheck::context::{GenContext};

pub fn ints() -> Strategy<Int> {
    Strategy { gen: gen, shrink: shrink }  -- Must work
}
```

## 5. Implementation Notes

### Files to Modify

| File | Change |
|------|--------|
| `crates/ash-engine/src/module_loader.rs` | Import-first visibility collection, opaque callable-signature type summaries, constructor-leak diagnostics |
| `crates/ash-engine/src/lib.rs` | Registers imported type identities before local module summary validation in `Engine::check_module_file` |
| `crates/ash-engine/tests/task_1540_type_annotation_quirks.rs` | Acceptance regressions for C90-1 through C90-5 |

### Order of Changes

1. Resolve imports before local public API/type validation in the engine module-loader.
2. Register imported type identities and selected opaque callable-signature identities before local summary validation.
3. Treat imported public types and imported callable-signature types as known in type definitions and callable annotations.
4. Diagnose unresolved signature types with import hints when a sibling module exports the missing type.
5. Verify focused Phase 154 regressions and engine/typeck gates.

## 6. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-057 | Amends: type module pipeline |
| SPEC-072 | Consistent: callable type annotations |
| ASSESSMENT-002 | Builds on: analysis of type annotation quirks |

## 7. Closeout Criteria

- [x] C90-1 through C90-5 all pass
- [x] No regressions in existing type tests
- [x] PLAN-154 and PLAN-INDEX updated
- [x] CHANGELOG.md records the fix
