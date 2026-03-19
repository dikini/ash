# TASK-136: Option and Result Standard Library

## Status: ✅ Complete

## Description

Implement standard library modules for Option and Result types with helper functions.

## Specification Reference

- SPEC-020: ADT Types - Section 9.1, 9.2

## Requirements

Implement built-in modules:
```ash
-- Option module
pub type Option<T> = Some { value: T } | None;
pub fn is_some<T>(opt: Option<T>) -> Bool;
pub fn is_none<T>(opt: Option<T>) -> Bool;
pub fn unwrap<T>(opt: Option<T>) -> T;
pub fn map<T, U>(opt: Option<T>, f: Fun(T) -> U) -> Option<U>;

-- Result module
pub type Result<T, E> = Ok { value: T } | Err { error: E };
pub fn is_ok<T, E>(res: Result<T, E>) -> Bool;
pub fn map<T, E, U>(res: Result<T, E>, f: Fun(T) -> U) -> Result<U, E>;
```

## TDD Steps

### Step 1: Create Standard Library Directory

**Directory**: `std/`

```bash
mkdir -p std/src
```

### Step 2: Create Option Module

**File**: `std/src/option.ash`

```ash
-- Option type and helper functions
-- 
-- Option<T> represents an optional value: either Some(T) or None.

pub type Option<T> =
    | Some { value: T }
    | None;

-- Returns true if the option is Some
pub fn is_some<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => true,
        None => false
    }
}

-- Returns true if the option is None
pub fn is_none<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => false,
        None => true
    }
}

-- Returns the value if Some, otherwise panics
pub fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        Some { value: v } => v,
        None => panic "called `unwrap` on a `None` value"
    }
}

-- Returns the value if Some, otherwise returns default
pub fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some { value: v } => v,
        None => default
    }
}

-- Maps Option<T> to Option<U> by applying a function
pub fn map<T, U>(opt: Option<T>, f: Fun(T) -> U) -> Option<U> {
    match opt {
        Some { value: v } => Some { value: f(v) },
        None => None
    }
}

-- Returns None if the option is None, otherwise returns optb
pub fn and<T>(opt: Option<T>, optb: Option<T>) -> Option<T> {
    match opt {
        Some { value: _ } => optb,
        None => None
    }
}

-- Returns the option if it contains a value, otherwise returns optb
pub fn or<T>(opt: Option<T>, optb: Option<T>) -> Option<T> {
    match opt {
        Some { value: _ } => opt,
        None => optb
    }
}

-- Converts Option<T> to Result<T, E>
pub fn ok_or<T, E>(opt: Option<T>, err: E) -> Result<T, E> {
    match opt {
        Some { value: v } => Ok { value: v },
        None => Err { error: err }
    }
}
```

### Step 3: Create Result Module

**File**: `std/src/result.ash`

```ash
-- Result type and helper functions
--
-- Result<T, E> represents either success (Ok) or failure (Err).

pub type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };

-- Returns true if the result is Ok
pub fn is_ok<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => true,
        Err { error: _ } => false
    }
}

-- Returns true if the result is Err
pub fn is_err<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => false,
        Err { error: _ } => true
    }
}

-- Returns the value if Ok, otherwise panics
pub fn unwrap<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: e } => panic "called `unwrap` on an `Err` value"
    }
}

-- Returns the error if Err, otherwise panics
pub fn unwrap_err<T, E>(res: Result<T, E>) -> E {
    match res {
        Ok { value: _ } => panic "called `unwrap_err` on an `Ok` value",
        Err { error: e } => e
    }
}

-- Returns the value if Ok, otherwise returns default
pub fn unwrap_or<T, E>(res: Result<T, E>, default: T) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: _ } => default
    }
}

-- Maps Result<T, E> to Result<U, E> by applying a function to Ok
pub fn map<T, E, U>(res: Result<T, E>, f: Fun(T) -> U) -> Result<U, E> {
    match res {
        Ok { value: v } => Ok { value: f(v) },
        Err { error: e } => Err { error: e }
    }
}

-- Maps Result<T, E> to Result<T, F> by applying a function to Err
pub fn map_err<T, E, F>(res: Result<T, E>, f: Fun(E) -> F) -> Result<T, F> {
    match res {
        Ok { value: v } => Ok { value: v },
        Err { error: e } => Err { error: f(e) }
    }
}

-- Chains operations that return Result
pub fn and_then<T, E, U>(res: Result<T, E>, f: Fun(T) -> Result<U, E>) -> Result<U, E> {
    match res {
        Ok { value: v } => f(v),
        Err { error: e } => Err { error: e }
    }
}

-- Converts Result<T, E> to Option<T>
pub fn ok<T, E>(res: Result<T, E>) -> Option<T> {
    match res {
        Ok { value: v } => Some { value: v },
        Err { error: _ } => None
    }
}

-- Converts Result<T, E> to Option<E>
pub fn err<T, E>(res: Result<T, E>) -> Option<E> {
    match res {
        Ok { value: _ } => None,
        Err { error: e } => Some { value: e }
    }
}
```

### Step 4: Create Prelude

**File**: `std/src/prelude.ash`

```ash
-- Standard library prelude
-- Automatically imported in all modules

use option::{Option, Some, None};
use result::{Result, Ok, Err};

-- Re-export commonly used functions
pub use option::{is_some, is_none, unwrap, unwrap_or, map as map_opt};
pub use result::{is_ok, is_err, unwrap as unwrap_res, unwrap_or as unwrap_or_res};
```

### Step 5: Register Built-in Types

**File**: `crates/ash-typeck/src/type_env.rs`

Update `add_builtin_types`:

```rust
fn add_builtin_types(&mut self) {
    // Option<T>
    let option_def = TypeDef {
        name: "Option".into(),
        params: vec![TypeVar(0)],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".into(),
                fields: vec![("value".into(), TypeExpr::Var(TypeVar(0)))],
            },
            VariantDef {
                name: "None".into(),
                fields: vec![],
            },
        ]),
        visibility: Visibility::Public,
    };
    self.register_type(option_def);
    
    // Result<T, E>
    let result_def = TypeDef {
        name: "Result".into(),
        params: vec![TypeVar(0), TypeVar(1)],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Var(TypeVar(0)))],
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Var(TypeVar(1)))],
            },
        ]),
        visibility: Visibility::Public,
    };
    self.register_type(result_def);
}
```

### Step 6: Create Integration Tests

**File**: `tests/adt_stdlib.ash`

```ash
-- Test Option helpers
workflow test_option {
    let some = Some { value: 42 };
    let none: Option<Int> = None;
    
    observe test with {} as _ {
        assert is_some(some);
        assert !is_some(none);
        assert !is_none(some);
        assert is_none(none);
        
        assert unwrap(some) == 42;
        assert unwrap_or(none, 0) == 0;
        
        let doubled = map(some, fn(x) => x * 2);
        assert unwrap(doubled) == 84;
    }
    
    ret Done;
}

-- Test Result helpers
workflow test_result {
    let ok = Ok { value: 42 };
    let err = Err { error: "oops" };
    
    observe test with {} as _ {
        assert is_ok(ok);
        assert !is_ok(err);
        assert !is_err(ok);
        assert is_err(err);
        
        assert unwrap(ok) == 42;
        assert unwrap_or(err, 0) == 0;
    }
    
    ret Done;
}

-- Test with control links
workflow supervisor {
    spawn worker with {} as w;
    let (w_addr, w_ctrl) = split w;
    
    -- w_ctrl: Option<ControlLink<Worker>>
    if is_some(w_ctrl) then {
        act log with "Have control link";
    } else {
        act log with "No control link";
    }
    
    ret Done;
}
```

### Step 7: Run Tests

```bash
# Run type checker tests
cargo test -p ash-typeck option -- --nocapture
cargo test -p ash-typeck result -- --nocapture

# Run integration tests
cargo test -p ash adt_stdlib -- --nocapture
```

## Completion Checklist

- [ ] Option type defined in std
- [ ] Result type defined in std
- [ ] Helper functions: is_some, is_none, is_ok, is_err
- [ ] Helper functions: unwrap, unwrap_or, unwrap_err
- [ ] Helper functions: map, map_err, and_then
- [ ] Helper functions: and, or, ok_or, ok, err
- [ ] Built-in types registered in type environment
- [ ] Prelude with re-exports
- [ ] Integration tests for Option
- [ ] Integration tests for Result
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-127 (Type Check Constructors)
- TASK-128 (Type Check Patterns)
- TASK-134 (Spawn Option ControlLink)

## Blocked By

- TASK-127
- TASK-128
- TASK-134

## Blocks

- Documentation updates
- Example workflows using Option/Result
