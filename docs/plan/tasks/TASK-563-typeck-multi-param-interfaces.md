# TASK-563: Type Checker — Multi-Parameter Interfaces and Impl Registry Redesign

**Phase:** 83
**Spec:** SPEC-033 §5
**Related:** TASK-562, TASK-565
**Estimate:** 4 hours
**Status:** ✅ Complete

## Description

Remove the single type-parameter restriction on interfaces and concrete impl blocks. Redesign the impl registry so that impls are keyed by the full interface application (e.g., `Map<String, Int>`) rather than a single bare type.

> **Registry continuity:** This task changes the `HashMap` key from `(interface, bare_type)` to `(interface, full_application)`. TASK-565 (Phase 83, next) will migrate the storage from `HashMap` to `Vec<ImplScheme>`. Keeping the key change here ensures multi-parameter interfaces work before generic schemes are introduced.

## Requirements

### Functional Requirements

1. `register_interface` accepts any number of type parameters.
2. `register_impl` validates that the number of type arguments matches the interface's type parameter count.
3. Impls are stored with the key `(interface_name, Type::Constructor { name: interface_name, args: ... })`.
4. `resolve_interface_method_call` constructs the impl head from all interface type parameters after unification.
5. Underdetermined interface type parameters (e.g., `From::from("hello")` where only `A` is known) produce a type error unless context resolves them.
6. `DuplicateImpl` and `MissingImpl` error messages report the full interface application.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task563_pair_two_param_interface_registers() {
    use ash_parser::surface::{InterfaceDef, InterfaceMethodSig, SurfaceType, Visibility};
    let mut env = TypeEnv::with_builtin_types();
    let pair_iface = InterfaceDef {
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        methods: vec![
            InterfaceMethodSig {
                name: "first".into(),
                params: vec![
                    SurfaceType::Constructor {
                        name: "Pair".into(),
                        args: vec![
                            SurfaceType::Name("A".into()),
                            SurfaceType::Name("B".into()),
                        ],
                    }
                ],
                return_type: SurfaceType::Name("A".into()),
                span: Span::default(),
            }
        ],
        visibility: Visibility::Inherited,
        span: Span::default(),
    };
    env.register_interface(&pair_iface).expect("register Pair");
    assert!(env.lookup_interface("Pair").is_some());
}

#[test]
fn task563_concrete_multi_param_impl_resolves() {
    // Register Pair<A,B> and impl Pair<Int,String>
    // Call Pair::first(my_pair)
    // Assert return type is Int
}

#[test]
fn task563_from_underdetermined_param_errors() {
    // Register From<A,B> { from(A) -> B }
    // Call From::from("hello") without context for B
    // Assert type error
}
```

> **Note:** These tests use surface `InterfaceDef` and `SurfaceType` because `TypeEnv::register_interface` accepts the surface AST directly. Do not mix internal `ash_typeck::Type` with surface `InterfaceMethodSig`.

### Step 2: Registry Redesign

**File:** `crates/ash-typeck/src/type_env.rs`

Remove from `register_impl`:
```rust
if interface_info.type_params.len() != 1 || def.type_args.len() != 1 {
    return Err(TypeEnvError::InvalidDefinition(
        "closed-world interface MVP only supports single-parameter ...".to_string()
    ));
}
```

Replace with arity check:
```rust
if interface_info.type_params.len() != def.type_args.len() {
    return Err(TypeEnvError::InvalidDefinition(format!(
        "interface '{}' expects {} type parameters, but impl provides {}",
        interface_name,
        interface_info.type_params.len(),
        def.type_args.len()
    )));
}
```

Change impl key construction:
```rust
let impl_head = Type::Constructor {
    name: QualifiedName::root(interface_name),
    args: lowered_type_args,
    kind: Kind::Type,
};
```

Update `DuplicateImpl` and `MissingImpl` messages to use the full interface application string.

### Step 3: Method Resolution

**File:** `crates/ash-typeck/src/type_env.rs`

In `resolve_interface_method_call`, after zip-unifying parameters:

```rust
let head_args: Vec<Type> = method_info.type_params.iter()
    .map(|tp| subst.apply(&Type::Var(*tp)))
    .collect();

let target_head = Type::Constructor {
    name: QualifiedName::root(interface.to_string()),
    args: head_args,
    kind: Kind::Type,
};

// Look up impl using target_head
```

Add a check: if any `head_args` still contains an unresolved `Type::Var`, return an error indicating underdetermined parameters.

## Verification Steps

- [x] `cargo test -p ash-typeck task563` passes
- [x] All existing single-parameter interface tests still pass
- [x] `cargo clippy -p ash-typeck --all-targets --all-features` clean
