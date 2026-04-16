# TASK-565: Type Checker — Generic Impl Schemes, Overlap Checking, and Recursive Resolution

**Phase:** 83
**Spec:** SPEC-034 §5
**Related:** TASK-563 (Complete), TASK-564 (blocked until parser/AST lands)
**Estimate:** 6 hours
**Status:** 📝 Planned (blocked: requires TASK-564 AST changes)

## Description

Replace the concrete impl map with a scheme-based registry. Implement ordered scheme search, overlap rejection, and recursive `where` bound checking with a depth limit.

> **Sequencing note:** This task is blocked until TASK-564 lands the new AST fields (`type_params`, `where_bounds`, `associated_types`, `associated_type_bindings`). The registry rewrite from `HashMap<(String, Type), ImplInfo>` to `Vec<ImplScheme>` is the highest-risk structural change in Phase 83 and should not be interleaved with other work.

> **Bound-checking limitation:** For this MVP, `where` bounds may only reference single-parameter interfaces (e.g., `T: Serialize`). Multi-parameter interfaces in `where` clauses (e.g., `T: Map<K, V>`) are deferred. The bound-head construction in `find_matching_impl_scheme` assumes a single type argument.

## Requirements

### Functional Requirements

1. `TypeEnv` stores `impls: Vec<ImplScheme>` instead of `HashMap<(String, Type), ImplInfo>`.
2. Every concrete and generic impl lowers to an `ImplScheme` whose head is the full interface application.
3. `register_impl` rejects overlapping schemes for the same interface.
4. `resolve_interface_method_call` searches schemes in declaration order, unifies the target head, and recursively checks `where` bounds.
5. Recursive bound checking stops with `RecursiveBound` error if depth exceeds 32.
6. `WrongArity` error added for interface method call argument count mismatches.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task565_generic_impl_scheme_registers() {
    // Register Serialize<T> interface
    // Register impl<T> Serialize<List<T>> where T: Serialize
    // Assert impls list contains 1 scheme with type_params=[T]
}

#[test]
fn task565_overlapping_impls_rejected() {
    // Register impl<T> Serialize<List<T>>
    // Attempt to register impl<T> Serialize<List<T>> again
    // Assert OverlappingImpls error
}

#[test]
fn task565_recursive_where_bound_resolution() {
    // Register Serialize<T> and impl<T> Serialize<List<T>> where T: Serialize
    // Call Serialize::serialize(nested_list, s) where nested_list: List<List<Int>>
    // Assert resolution succeeds by recursive bound checking
}

#[test]
fn task565_recursive_bound_depth_limit_errors() {
    // Create a cyclic generic impl (e.g., A<T> where T: A<T>)
    // Assert RecursiveBound error after depth 32
}
```

### Step 2: Data Structures

**File:** `crates/ash-typeck/src/type_env.rs`

Define:
```rust
pub struct ImplScheme {
    pub interface: String,
    pub type_params: Vec<TypeVar>,
    pub head: Type,               // e.g., Serialize<List<T>>
    pub where_bounds: Vec<WhereBound>,
    pub methods: Vec<ImplMethodInfo>,
}

pub struct SelectedScheme {
    pub substitution: Substitution,
}
```

Update `TypeEnv`:
```rust
impls: Vec<ImplScheme>,
```

### Step 3: Registration

**File:** `crates/ash-typeck/src/type_env.rs`

Update `register_impl`:
1. Build `head` as `Type::Constructor { name: interface_name, args: lowered_type_args, kind: Kind::Type }`.
2. Check overlap: iterate existing schemes for same interface; if `try_unify(&existing.head, &new_head).is_ok()`, return `OverlappingImpls`.
3. Push new `ImplScheme`.

Add error variants:
```rust
#[error("overlapping impls for interface '{interface}'")]
OverlappingImpls { interface: String },

#[error("recursive interface bound exceeded depth limit")]
RecursiveBound { message: String },
```

### Step 4: Resolution Algorithm

**File:** `crates/ash-typeck/src/type_env.rs`

Update `resolve_interface_method_call`:
1. Unify method params with arg types (zip, as in TASK-562).
2. Construct `target_head` from all interface type params.
3. Call `find_matching_impl_scheme(interface, &target_head, 0)`.
4. Apply selected substitution to return type and return.

Add `find_matching_impl_scheme`:
```rust
fn find_matching_impl_scheme(
    &self,
    interface: &str,
    target_head: &Type,
    depth: usize,
) -> Result<SelectedScheme, TypeEnvError> {
    if depth > 32 {
        return Err(TypeEnvError::RecursiveBound { message: "depth limit".into() });
    }
    for scheme in self.impls.iter().filter(|s| s.interface == interface) {
        if let Ok(subst) = try_unify(&scheme.head, target_head) {
            let mut bounds_ok = true;
            for bound in &scheme.where_bounds {
                let bounded_ty = subst.apply(&Type::Var(bound.type_var));
                let bound_head = Type::Constructor {
                    name: QualifiedName::root(bound.interface.clone()),
                    args: vec![bounded_ty],
                    kind: Kind::Type,
                };
                if self.find_matching_impl_scheme(&bound.interface, &bound_head, depth + 1).is_err() {
                    bounds_ok = false;
                    break;
                }
            }
            if bounds_ok {
                return Ok(SelectedScheme { substitution: subst });
            }
        }
    }
    Err(TypeEnvError::MissingImpl { interface: interface.into(), ty: target_head.to_string() })
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck task565` passes
- [ ] `cargo clippy -p ash-typeck --all-targets --all-features` clean
