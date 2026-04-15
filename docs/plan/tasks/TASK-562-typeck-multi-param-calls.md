# TASK-562: Type Checker/Interpreter — Multi-Parameter Interface Call Resolution

**Phase:** 82
**Spec:** SPEC-032 §5-6
**Related:** TASK-561
**Estimate:** 5 hours
**Status:** ✅ Complete

## Description

Teach the type checker to resolve multi-parameter interface method calls through `Expr::Call`, zip-unifying all argument types against the method signature. Clean up all `InterfaceMethodCall` references from the type checker and interpreter.

> **Call-site enumeration:** `resolve_interface_method_call` is called from:
> - `crates/ash-typeck/src/check_expr.rs:263`
> - `crates/ash-typeck/src/lib.rs:483`
> - `crates/ash-typeck/src/purity.rs:101`
> All three must be updated when the signature changes from `&Type` to `&[Type]`.

## Requirements

### Functional Requirements

1. `Expr::Call { module: Some(interface_name), func, args }` checks whether `interface_name` is a registered interface.
2. If it is an interface, `resolve_interface_method_call` receives all argument types and zip-unifies them against the method parameters.
3. `resolve_interface_method_call` returns an arity error if argument count mismatches parameter count.
4. All `InterfaceMethodCall` handling is removed from `check_expr.rs`, `lib.rs`, `purity.rs`, `names.rs`, and `eval.rs`.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task562_eq_two_param_call_typechecks() {
    let mut env = TypeEnv::with_builtin_types();
    // Register interface Eq<T> { eq(T, T) -> Bool }
    // Register impl Eq<Int> { eq(a, b) = a == b }
    // (setup omitted for brevity)
    let call = Expr::Call {
        func: "eq".into(),
        module: Some("Eq".into()),
        args: vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
        ],
        span: Span::default(),
    };
    let result = check_expr(&env, &call);
    assert!(result.is_ok());
    assert_eq!(result.substitution.apply(&result.ty), Type::Bool);
}

#[test]
fn task562_eq_arity_mismatch_errors() {
    let mut env = TypeEnv::with_builtin_types();
    let call = Expr::Call {
        func: "eq".into(),
        module: Some("Eq".into()),
        args: vec![Expr::Literal(Literal::Int(1))], // only 1 arg
        span: Span::default(),
    };
    let result = check_expr(&env, &call);
    assert!(!result.is_ok(), "arity mismatch must fail");
}
```

### Step 2: Type Checker Changes

**File:** `crates/ash-typeck/src/type_env.rs`

Remove:
```rust
if method_info.params.len() != 1 { ... return Err(...); }
```

Update `resolve_interface_method_call` signature from:
```rust
pub fn resolve_interface_method_call(
    &self,
    interface: &str,
    method: &str,
    arg_type: &Type,
) -> Result<Type, TypeEnvError>
```

To:
```rust
pub fn resolve_interface_method_call(
    &self,
    interface: &str,
    method: &str,
    arg_types: &[Type],
) -> Result<Type, TypeEnvError>
```

Add zip-unification:
```rust
if method_info.params.len() != arg_types.len() {
    return Err(TypeEnvError::InvalidDefinition(format!(
        "expected {} arguments, found {}",
        method_info.params.len(),
        arg_types.len()
    )));
}
let mut subst = Substitution::new();
for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
    subst = unify(expected, actual).map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}")))?;
}
```

**File:** `crates/ash-typeck/src/check_expr.rs`

Update `Expr::Call` handling to detect interface calls and pass `arg_types` slice to `resolve_interface_method_call`.

Remove any `Expr::InterfaceMethodCall` match arm.

**Files:** `crates/ash-typeck/src/lib.rs`, `crates/ash-typeck/src/purity.rs`, `crates/ash-typeck/src/names.rs`

Remove `InterfaceMethodCall` handling.

### Step 3: Interpreter Cleanup

**File:** `crates/ash-interp/src/eval.rs`

Remove `Expr::InterfaceMethodCall` evaluation branch. It should no longer be reachable.

## Verification Steps

- [ ] `cargo test -p ash-typeck task562` passes
- [ ] `cargo test -p ash-interp` passes (no `InterfaceMethodCall` branches)
- [ ] `cargo clippy -p ash-typeck -p ash-interp --all-targets --all-features` clean
