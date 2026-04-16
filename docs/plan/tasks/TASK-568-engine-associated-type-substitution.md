# TASK-568: Engine — Associated Type Substitution in Monomorphized Bodies

**Phase:** 83
**Spec:** SPEC-035 §6
**Related:** TASK-566 (monomorphization), TASK-567 (associated type normalization)
**Estimate:** 3 hours
**Status:** 📝 Planned (blocked: requires TASK-566 and TASK-567)

## Description

Extend the engine monomorphization pass to substitute associated type projections (`S::Ok`) with their normalized concrete types inside instantiated impl method bodies. Ensure `Type::Associated` never appears at runtime.

## Requirements

### Functional Requirements

1. During monomorphization, after selecting an impl scheme and computing the substitution, normalize all `Type::Associated` occurrences in the method body using the scheme's associated type bindings.
2. The monomorphized body contains only concrete types (`Type::Constructor`, primitive types, etc.).
3. Interface method calls at runtime dispatch to fully concrete, associated-type-free code.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-engine/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task568_associated_type_replaced_in_monomorphized_body() {
    // Setup:
    //   interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok }
    //   impl Serializer<JsonWriter> { type Ok = String; serialize_bool(w, v) = w.write(v) }
    // Compile a call to Serializer::serialize_bool(writer, true)
    //
    // Assert the monomorphized body has return type String and no Type::Associated nodes.
}

#[test]
fn task568_generic_impl_associated_type_substituted() {
    // Setup:
    //   impl<T> Serializer<List<T>> where T: Serialize {
    //       type Ok = List<String>
    //       serialize(items, s) = list::map(items, fn(x) { Serialize::serialize(x, s) })
    //   }
    // Compile Serialize::serialize(list_of_ints, s)
    // Assert the monomorphized Ok type is List<String>.
}
```

### Step 2: Extend Monomorphization Pass

**File:** `crates/ash-engine/src/monomorphize.rs`

Add associated-type normalization to the expression visitor:

```rust
pub fn normalize_expr_associated_types(
    expr: &mut Expr,
    scheme: &ImplScheme,
    subst: &Substitution,
    type_env: &TypeEnv,
) -> Result<(), TypeEnvError> {
    // Walk the expression tree.
    // For every node that carries an explicit type annotation:
    //   - FnDef return_type
    //   - Pattern::Typed
    //   - Constructor fields with explicit types (if any)
    // Replace Type::Associated with the normalized concrete type.
}
```

**File:** `crates/ash-engine/src/lib.rs`

In the post-typecheck monomorphization pipeline:

```rust
// After selecting scheme and substitution:
let mut body = scheme.get_method_body(method_name).clone();
type_env.normalize_associated_types_in_expr(&mut body, scheme, &subst)?;
// Then apply type-parameter substitution as in TASK-566.
// Finally emit the concrete body.
```

### Step 3: Runtime Assertion

Add a debug-only traversal in the engine that panics if any `Type::Associated` remains in core IR after monomorphization. This can be a simple `assert_no_associated_types(expr)` helper.

## Verification Steps

- [ ] `cargo test -p ash-engine task568` passes
- [ ] `cargo test --all` passes
- [ ] `cargo clippy -p ash-engine --all-targets --all-features` clean
