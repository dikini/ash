# TASK-551: Core IR -- FnDef, FnApply, EnvFrame, Closure value

**Phase:** 80
**Spec:** SPEC-031 §5.1, §5.2, §5.3, §10.1, §10.2, §10.3
**Depends on:** None
**Estimate:** 6 hours

## Description

Add the foundation types for first-class functions to the core IR and interpreter. This is non-breaking: new variants and types are added but nothing produces them yet.

## Requirements

### 1. ash-core/src/ast.rs

Add two new `Expr` variants:

```rust
/// Anonymous function definition (closure creation). [SPEC-031 §5.1]
FnDef {
    params: Vec<(Name, Option<Type>)>,
    return_type: Option<Type>,
    body: Box<Expr>,
},

/// Function application. [SPEC-031 §5.4]
FnApply {
    func: Box<Expr>,
    args: Vec<Expr>,
},
```

Update all match arms on `Expr` across the codebase (formatting, display, visiting, etc.) to handle these variants. They may return placeholder/unimplemented for now.

### 2. ash-core/src/env_frame.rs (NEW FILE)

Implement `EnvFrame` and `BindingSlot` per SPEC-031 §5.3:

- `BindingSlot::Bound(Value)` for normal bindings
- `BindingSlot::Late(Arc<Mutex<Option<Value>>>)` for recursive bindings
- `EnvFrame { bindings: HashMap<String, BindingSlot>, parent: Option<Arc<EnvFrame>> }`
- All types must be naturally `Send + Sync` (no `unsafe`)

### 3. ash-core/src/value.rs

Add `Value::Closure` variant per SPEC-031 §5.2:

```rust
Closure {
    params: Vec<(String, Option<String>)>,
    body: Box<Expr>,
    env: Arc<EnvFrame>,
},
```

Replace `#[derive(Serialize, Deserialize)]` with manual implementation. `Closure` serialization must produce a runtime error (no serialized form). All other variants serialize as before.

### 4. ash-core/src/lib.rs

Export the new `env_frame` module.

### 5. ash-interp/src/error.rs

Add new `EvalError` variants:
- `NotCallable { value: Value }` -- applied a non-closure value
- `BoundaryViolation { value: Value, context: String }` -- closure crossed three-vertex boundary (runtime fallback)

### 6. ash-interp/src/context.rs

Add two new methods:
- `to_env_frame(&self) -> Arc<EnvFrame>` -- snapshot current scope chain
- `from_env_frame(frame: &Arc<EnvFrame>) -> Self` -- create context from captured frame

### 7. ash-interp/src/eval.rs

Add eval cases for `Expr::FnDef` and `Expr::FnApply` per SPEC-031 §10.1.

Implement recursive let evaluation per SPEC-031 §10.3: create `BindingSlot::Late`, evaluate `FnDef`, fill the late binding.

### 8. Send + Sync Assertions

Add compile-time assertions:
```rust
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Value>();
    assert_send_sync::<Expr>();
};
```

## TDD Steps

1. Write property tests for `EnvFrame` capture: shared parent chain, `BindingSlot::Late` fill-then-resolve
2. Write tests for `Value::Closure` construction and application
3. Write test for recursive factorial
4. Write test for higher-order function (apply/double pattern)
5. Write test that serializing `Value::Closure` produces error
6. Verify `cargo test --all` passes (no existing tests broken)

## Completion Checklist

- [ ] `Expr::FnDef` and `Expr::FnApply` in `ast.rs` with all match arms updated
- [ ] `EnvFrame` and `BindingSlot` in new `env_frame.rs`
- [ ] `Value::Closure` in `value.rs` with manual serde (no serialize)
- [ ] `NotCallable` and `BoundaryViolation` error variants
- [ ] `Context::to_env_frame` and `Context::from_env_frame`
- [ ] `eval_expr` handles `FnDef` and `FnApply`
- [ ] Recursive let evaluation via `BindingSlot::Late`
- [ ] Send + Sync compile-time assertions pass
- [ ] Property tests for env capture, closures, recursion, higher-order
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
