# SPEC-090: Fix Type Annotation Quirks in fn Expressions with Imported Types

**Status:** Draft
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

Distinguish between **type name visibility** and **constructor visibility**:

```ash
// internal.ash
pub type Secret = Secret { value: Int };  -- Type public, constructor private

fn make_secret(v: Int) -> Secret {        -- Private smart constructor
    Secret { value: v }
}

// public.ash
use internal::{Secret};  -- Imports type name, NOT constructor

pub fn double_secret(s: Secret) -> Secret {
    -- Can use Secret in signatures (type is imported)
    -- Cannot construct Secret (constructor not imported)
    make_secret(s.value * 2)  -- Must use smart constructor
}
```

### 3.4 Type Inference Leakage Prevention

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

### C90-3: Smart constructors

```ash
// internal.ash
pub type Secret = Secret { value: Int };

fn make_secret(v: Int) -> Secret {  -- Private constructor
    Secret { value: v }
}

// public.ash
use internal::{Secret};

pub fn double(s: Secret) -> Secret {
    make_secret(s.value * 2)  -- Must work
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
| `crates/ash-parser/src/parse_module.rs` | Two-pass processing: imports first, then types |
| `crates/ash-typeck/src/type_env.rs` | Register imported types before local types |
| `crates/ash-typeck/src/check.rs` | Type name resolution with imported types |
| `crates/ash-typeck/src/diagnostics.rs` | Better error messages for type leakage |

### Order of Changes

1. Modify parser to collect imports before type definitions
2. Modify TypeEnv to register imported types early
3. Update type name resolution to check imported types
4. Add diagnostic for type inference leakage
5. Verify all tests pass

## 6. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-057 | Amends: type module pipeline |
| SPEC-072 | Consistent: callable type annotations |
| ASSESSMENT-002 | Builds on: analysis of type annotation quirks |

## 7. Closeout Criteria

- [ ] C90-1 through C90-5 all pass
- [ ] No regressions in existing type tests
- [ ] PLAN-154 and PLAN-INDEX updated
- [ ] CHANGELOG.md records the fix
