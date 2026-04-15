# TASK-561: Parser/AST — Multi-Parameter Method Signatures and Impl Definitions

**Phase:** 82
**Spec:** SPEC-032 §4
**Related:** TASK-422, TASK-562
**Estimate:** 4 hours
**Status:** ✅ Complete

## Description

Update the surface and core AST to support multiple parameters in interface method signatures and impl method definitions. Remove the deprecated `InterfaceMethodCall` AST node and update all parsers and lowering accordingly.

## Requirements

### Functional Requirements

1. `ImplMethodDef.param: Name` becomes `ImplMethodDef.params: Vec<Name>` in both surface and core AST.
2. Interface method signatures parse any number of parameters: `name(Type1, Type2, ...) -> ReturnType`.
3. Impl method definitions parse any number of parameters: `name(p1, p2, ...) = expr`.
4. `Expr::InterfaceMethodCall` is removed from surface and core AST.
5. Lowering rejects no special cases for interface method calls (they lower as ordinary `Expr::Call`).

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-parser/tests/closed_world_interfaces_task_422.rs` (or new parser test file)

> **Compilation order:** The tests below reference `params` (plural) on `InterfaceMethodDef` and `ImplMethodDef`. These tests will not compile until Step 2 (AST changes) is applied. This is intentional TDD — write the tests, watch them fail to compile, then make the AST changes.

**Tests:**

```rust
#[test]
fn task561_multi_param_interface_method_signature_parses() {
    let input = r#"interface Eq<T> { eq(T, T) -> Bool }"#;
    let mut parse_input = new_input(input);
    let def = parse_module::parse_interface_definition(&mut parse_input).unwrap();
    match def {
        Definition::Interface(iface) => {
            assert_eq!(iface.methods.len(), 1);
            assert_eq!(iface.methods[0].params.len(), 2, "eq should take 2 params");
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn task561_multi_param_impl_method_definition_parses() {
    let input = r#"impl Eq<Int> { eq(a, b) = a == b }"#;
    let mut parse_input = new_input(input);
    let def = parse_module::parse_impl_definition(&mut parse_input).unwrap();
    match def {
        Definition::Impl(impl_def) => {
            assert_eq!(impl_def.methods.len(), 1);
            assert_eq!(impl_def.methods[0].params.len(), 2, "eq impl should have 2 params");
        }
        _ => panic!("expected impl"),
    }
}

#[test]
fn task561_interface_method_call_routes_through_expr_call() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input("Eq::eq(x, y)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call { func, module, args, .. } => {
            assert_eq!(func.as_ref(), "eq");
            assert_eq!(module.as_ref().map(|n| n.as_ref()), Some("Eq"));
            assert_eq!(args.len(), 2);
        }
        other => panic!("expected Expr::Call, got {other:?}"),
    }
}
```

### Step 2: AST Changes

**File:** `crates/ash-parser/src/surface.rs`

Change:
```rust
pub struct ImplMethodDef {
    pub name: Name,
    pub param: Name,        // -- OLD
    pub body: Expr,
    pub span: Span,
}
```

To:
```rust
pub struct ImplMethodDef {
    pub name: Name,
    pub params: Vec<Name>,  // -- NEW
    pub body: Expr,
    pub span: Span,
}
```

Remove `InterfaceMethodCall` from `Expr` enum.

**File:** `crates/ash-core/src/ast.rs`

Mirror both changes.

### Step 3: Parser Implementation

**File:** `crates/ash-parser/src/parse_module.rs`

Update `parse_interface_method_signature` to parse a parenthesized, comma-separated list of types before `-> ReturnType`.

Update `parse_impl_method_definition` to parse a parenthesized, comma-separated list of parameter names.

**File:** `crates/ash-parser/src/parse_expr.rs`

Remove the `InterfaceMethodCall` branch. `Name::Name(args...)` already produces `Expr::Call { module: Some(...), ... }`; ensure this is the only path.

### Step 4: Lowering

**File:** `crates/ash-parser/src/lower.rs`

- Update `lower_impl_method_def` to produce `Vec<String>` params.
- Remove `InterfaceMethodCall` lowering rejection.
- Remove `InterfaceMethodCallNotSupported` from `LoweringError` if it becomes unused.

## Verification Steps

- [ ] `cargo test -p ash-parser task561` passes
- [ ] `cargo check --all` passes after AST changes
- [ ] `cargo clippy -p ash-parser --all-targets --all-features` clean
