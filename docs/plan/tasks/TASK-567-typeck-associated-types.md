# TASK-567: Type Checker — `Type::Associated`, Normalization, and Rigid Projections

**Phase:** 83
**Spec:** SPEC-035 §5
**Related:** TASK-564 (parser/AST), TASK-565 (scheme registry)
**Estimate:** 6 hours
**Status:** 📝 Planned (blocked: requires TASK-565)

## Description

Add associated type projections to the internal type representation, implement normalization after impl selection, and enforce the rigid-projection rule for unresolved associated types inside generic functions.

## Requirements

### Functional Requirements

1. `Type::Associated { interface, base, name }` represents projections like `S::Ok`.
2. `register_interface` stores `associated_types: Vec<String>` in `InterfaceInfo`.
3. `register_impl` validates that every associated type has exactly one binding and rejects extraneous ones.
4. `normalize_associated_types(ty, scheme, subst)` recursively replaces `Type::Associated` with the concrete binding from the selected impl scheme.
5. `resolve_interface_method_call` normalizes the return type before returning it.
6. Unification normalizes both sides before comparing when a scheme is known.
7. In generic code where no scheme is selected yet, identical rigid projections unify; a rigid projection does not unify with an arbitrary concrete type.
8. Error variants: `MissingAssociatedType`, `MismatchedProjectionInterface`, `AmbiguousAssociatedType`.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task567_associated_type_normalizes_in_return_type() {
    // Register Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok }
    // Register impl Serializer<JsonWriter> { type Ok = String; ... }
    // Call Serializer::serialize_bool(writer, true)
    // Assert return type is String (normalized from S::Ok)
}

#[test]
fn task567_rigid_projection_unifies_with_itself() {
    // In generic context fn<T: Serializer>(a: T::Ok, b: T::Ok) { }
    // Assert a and b unify (same rigid projection)
}

#[test]
fn task567_rigid_projection_rejects_concrete_match() {
    // fn<T: Serializer>(a: T::Ok) -> String { a }
    // Assert type error: T::Ok does not unify with String
}

#[test]
fn task567_missing_associated_type_in_impl_errors() {
    // impl Serializer<JsonWriter> { /* missing type Ok = ... */ serialize_bool(...) = ... }
    // Assert MissingAssociatedType at registration
}
```

### Step 2: Type Representation

**File:** `crates/ash-typeck/src/types.rs`

Add variant:
```rust
pub enum Type {
    // ... existing variants
    Associated {
        interface: String,
        base: Box<Type>,
        name: String,
    },
}
```

### Step 3: Registry Metadata

**File:** `crates/ash-typeck/src/type_env.rs`

Update `InterfaceInfo`:
```rust
pub struct InterfaceInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub associated_types: Vec<String>,   // NEW
    pub methods: HashMap<String, InterfaceMethodInfo>,
}
```

Update `ImplScheme`:
```rust
pub struct ImplScheme {
    pub interface: String,
    pub type_params: Vec<TypeVar>,
    pub head: Type,
    pub where_bounds: Vec<WhereBound>,
    pub associated_type_bindings: HashMap<String, Type>, // NEW
    pub methods: Vec<ImplMethodInfo>,
}
```

### Step 4: Normalization

**File:** `crates/ash-typeck/src/type_env.rs`

Implement:
```rust
pub fn normalize_associated_types(
    &self,
    ty: &Type,
    scheme: &ImplScheme,
    subst: &Substitution,
) -> Result<Type, TypeEnvError> {
    match ty {
        Type::Associated { interface, base, name } => {
            if scheme.interface != *interface {
                return Err(TypeEnvError::MismatchedProjectionInterface { ... });
            }
            let binding = scheme.associated_type_bindings.get(name)
                .ok_or_else(|| TypeEnvError::MissingAssociatedType { ... })?;
            let normalized = subst.apply(binding);
            self.normalize_associated_types(&normalized, scheme, subst)
        }
        Type::Constructor { name, args, kind } => {
            let norm_args = args.iter()
                .map(|a| self.normalize_associated_types(a, scheme, subst))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Constructor { name: name.clone(), args: norm_args, kind: kind.clone() })
        }
        Type::Fun(params, ret, effect) => {
            let norm_params = params.iter()
                .map(|p| self.normalize_associated_types(p, scheme, subst))
                .collect::<Result<Vec<_>, _>>()?;
            let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
            Ok(Type::Fun(norm_params, Box::new(norm_ret), *effect))
        }
        Type::Fn(params, ret) => {
            let norm_params = params.iter()
                .map(|p| self.normalize_associated_types(p, scheme, subst))
                .collect::<Result<Vec<_>, _>>()?;
            let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
            Ok(Type::Fn(norm_params, Box::new(norm_ret)))
        }
        Type::List(inner) => Ok(Type::List(Box::new(
            self.normalize_associated_types(inner, scheme, subst)?
        ))),
        Type::Record(fields) => {
            let norm_fields = fields.iter()
                .map(|(n, t)| Ok((n.clone(), self.normalize_associated_types(t, scheme, subst)?)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(norm_fields))
        }
        other => Ok(other.clone()),
    }
}
```

### Step 5: Rigid Projection Rule

**File:** `crates/ash-typeck/src/types.rs` (in `unify`)

When unifying two `Type::Associated` values:
- If `interface`, `base`, and `name` are identical, they unify with an empty substitution.
- Otherwise, unification fails.

When unifying `Type::Associated` with any other type:
- If the associated type cannot be normalized (because no scheme is in scope), unification fails.
- If it can be normalized, normalize first and retry.

For now, the simplest implementation is to let `Type::Associated` fail unification against anything except an identical `Type::Associated`. Normalization happens at the call site before unification is invoked.

## Verification Steps

- [ ] `cargo test -p ash-typeck task567` passes
- [ ] `cargo clippy -p ash-typeck --all-targets --all-features` clean
