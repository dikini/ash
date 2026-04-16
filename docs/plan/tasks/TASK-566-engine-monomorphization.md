# TASK-566: Engine — Post-Typecheck Monomorphization Pass for Generic Impls

**Phase:** 83
**Spec:** SPEC-034 §6
**Related:** TASK-565 (scheme registry), TASK-567 (associated types)
**Estimate:** 6 hours
**Status:** 📝 Planned (blocked: requires TASK-565)

## Description

Add a post-typecheck lowering pass in `ash-engine` that instantiates generic impl bodies at concrete call sites. The monomorphized method body replaces type parameters with the concrete types selected during interface resolution.

> **Pipeline stage note:** The engine currently lacks a post-typecheck lowering phase. This task requires establishing a new pipeline stage between type-checking and execution.
>
> **Recommended hook point:** `Engine::compile` (or the existing `Engine::check` → `Engine::execute` boundary) should gain an explicit lowering step:
> ```
> parse → type_check → monomorphize → execute
> ```
> Specifically:
> 1. After `type_check_module` returns a `TypeEnv`, walk the core AST (`CoreWorkflow` / `CoreExpr`)
>    and identify every `Expr::Call` with `module: Some(iface)`.
> 2. For each such call, invoke `type_env.resolve_interface_method_call` (or a new
>    `select_impl_scheme` API) to obtain the selected `ImplScheme` and substitution.
> 3. Clone the scheme's method body AST, apply the substitution, and replace the original
>    `Expr::Call` with the instantiated body (or a synthetic `Expr::FnApply` to a fresh
>    internal function).
> 4. The monomorphized module is then passed to the interpreter / executor.
>
> **File target:** `crates/ash-engine/src/monomorphize.rs` (new module) with a public entry
> point `monomorphize_module(module: &mut CoreModule, type_env: &TypeEnv) -> Result<...>`.
> This keeps the lowering logic isolated from `Engine`'s configuration and CLI surfaces.

## Requirements

### Functional Requirements

1. After type-checking resolves an interface call to a concrete `ImplScheme` and substitution, the engine produces a monomorphized method body.
2. Type-parameter substitution is applied to the impl method body AST.
3. The resulting core IR contains only concrete code — no generic impl dispatch at runtime.
4. Monomorphized bodies are either inlined into the call site or emitted as fresh internal functions.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-engine/tests/closed_world_interfaces_task_422.rs` (or new test file)

**Tests:**

```rust
#[test]
fn task566_generic_impl_monomorphizes_to_concrete_body() {
    // Build an engine with:
    //   interface Serialize<T> { serialize(T, Serializer) -> String }
    //   impl<T> Serialize<List<T>> where T: Serialize { serialize(items, s) = ... }
    //   impl Serialize<Int> { serialize(x, s) = ... }
    // Compile a call Serialize::serialize(my_list, s) where my_list: List<Int>
    // Assert the lowered core IR contains no generic parameters and references
    // the concrete List<Int> impl body.
}
```

### Step 2: Engine Integration

**File:** `crates/ash-engine/src/lib.rs`

Add a monomorphization step in the workflow lowering pipeline:

```rust
// Pseudo-code for the post-typecheck pass
fn monomorphize_interface_calls(
    workflow: &mut CoreWorkflow,
    type_env: &TypeEnv,
) -> Result<(), MonomorphizeError> {
    // Walk the workflow AST.
    // For every Expr::Call { module: Some(iface), func, args }:
    //   1. Resolve the interface method call via type_env to get the SelectedScheme.
    //   2. Retrieve the scheme's method body AST.
    //   3. Apply the substitution to the body (replace type parameters with concrete types).
    //   4. Replace the Expr::Call with the instantiated body (or a call to a fresh internal fn).
}
```

### Step 3: AST Substitution

**File:** `crates/ash-engine/src/monomorphize.rs` (new file)

Implement a type-substitution visitor for core AST expressions:

```rust
pub fn substitute_type_in_expr(expr: &mut Expr, subst: &Substitution) {
    // Recursively walk expr and apply substitution to any type annotations
    // inside Expr::FnDef, Pattern::Typed, etc.
}
```

This visitor only needs to touch nodes that carry explicit type information. Runtime values and control flow are unaffected.

### Step 4: Internal Function Naming

If emitting fresh internal functions rather than inlining:

```rust
fn monomorphized_name(interface: &str, method: &str, head: &Type) -> String {
    format!("{}::{}::{}_{}", interface, method, head)
}
```

These names are synthetic and not user-visible.

## Verification Steps

- [ ] `cargo test -p ash-engine task566` passes
- [ ] `cargo clippy -p ash-engine --all-targets --all-features` clean
