# SPEC-088: Closure Refinement and Effect-Safe Capture

**Status:** Implemented MVP (Phase 152); broader cross-stratum closure serialization remains out of scope
**Date:** 2026-06-17
**Amends:** [SPEC-031](SPEC-031-FIRST-CLASS-FUNCTIONS.md), [SPEC-072](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Builds on:** [SPEC-087](SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
**Plan:** [PLAN-152](../plan/PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## 1. Summary

Replace the blanket "no closures in pure functions" restriction with a precise capture-based rule:

> A closure created in context C may only capture values whose effect level ≤ C. A pure closure may not capture values produced by Act effects, capability handles, or closures with higher effect levels.

This allows pure closures that capture only pure data (e.g., `fn make_adder(n) { fn(x) { n + x } }`) while preserving the strict environment lattice.

## 2. Motivation

The current blanket ban (`ctx.is_pure()` rejects ALL closures) is overly conservative. It prevents natural patterns like:

```ash
fn make_adder(n) {
    fn(x) { n + x }  -- n is Int, pure value; should be allowed
}
```

The restriction was intended to prevent effect leakage, but it also rejects effect-free captures. A precise rule allows safe closures while maintaining the tower's integrity.

## 3. Normative Rule

### 3.1 Effect Level of Values

Every value has an effect level:

| Value Type | Effect Level | Examples |
|-----------|-------------|----------|
| Primitive literals | Pure | `42`, `"hello"`, `true` |
| Pure data constructors | Pure | `Point { x: 10, y: 20 }`, `Ok { value: 42 }` |
| Pure closures | Pure | `fn(x) { x + 1 }` (captures only pure values) |
| Effect-produced values | Act | `std::fs::read_file("/etc/passwd")` result |
| Capability handles | Act | `std::io::fs`, `std::process::process` |
| Act closures | Act | `\|x\| -*> { ... }` (reserved) |
| Proc closures | Proc | `\|x\| => { ... }` (reserved) |
| Workflow closures | Workflow | `\|x\| =*> { ... }` (reserved) |

### 3.2 Capture Rule

```text
For a closure created in context C:
  For each captured variable v:
    effect_level(v) ≤ effect_level(C)
```

Where:
- `effect_level(Pure) = 0`
- `effect_level(Act) = 1`
- `effect_level(Proc) = 2`
- `effect_level(Workflow) = 3`

### 3.3 Pure Closure Specifics

A closure created in a pure context (inside `fn`) may capture:
- ✅ Primitive values (`Int`, `String`, `Bool`)
- ✅ Pure record/enum values
- ✅ Other pure closures
- ✅ Type constructors (as values)

A pure closure may NOT capture:
- ❌ Capability handles (`std::io::fs`)
- ❌ Values produced by Act effects
- ❌ Act/Proc/Workflow closures (even as opaque values)
- ❌ Any value whose type contains `Type::Fun` (effectful callable)

## 4. Implementation Strategy

### 4.1 Static Analysis (Type-Checking Time)

The typechecker can enforce this rule by examining the types of captured variables:

```rust
// In closure creation (Expr::FnDef lowering)
fn check_closure_capture(env: &TypeEnv, captures: &[VarId], context: Context) -> Result<(), TypeError> {
    for var in captures {
        let ty = env.type_of(var);
        let effect = extract_effect_level(&ty);
        if effect > context.effect_level() {
            return Err(TypeError::CaptureEffectViolation {
                var: var.name(),
                var_effect: effect,
                context_effect: context.effect_level(),
            });
        }
    }
    Ok(())
}
```

### 4.2 Effect Extraction from Types

```rust
fn extract_effect_level(ty: &Type) -> EffectLevel {
    match ty {
        Type::Primitive(_) => EffectLevel::Pure,
        Type::Constructor(_) => EffectLevel::Pure,
        Type::Fn(_, _) => EffectLevel::Pure,  // pure callable
        Type::Fun(_, _, effect) => effect.level(),  // effectful callable
        Type::Capability(_) => EffectLevel::Act,  // capability handle
        Type::Act(_) => EffectLevel::Act,  // Act value
        Type::Proc(_) => EffectLevel::Proc,
        Type::Workflow(_) => EffectLevel::Workflow,
        _ => EffectLevel::Pure,  // default conservative
    }
}
```

### 4.3 Runtime Enforcement (Fallback)

If static analysis is incomplete, the runtime can enforce at closure creation:

```rust
// In eval.rs Expr::FnDef handling
let env_frame = ctx.to_env_frame();
if ctx.is_pure() {
    // Check that captured environment contains only pure values
    for (name, value) in env_frame.bindings() {
        if !value.is_pure() {
            return Err(EvalError::CaptureEffectViolation { ... });
        }
    }
}
```

## 5. Effect Leakage Scenarios Prevented

### 5.1 Capability Capture (Blocked)

```ash
workflow w {
    let fs = std::io::fs;           -- Act-level capability
    let bad = fn(path) {            -- ERROR: captures Act-level value in pure closure
        fs.read(path)
    };
    ret bad
}
```

**Error:** `CaptureEffectViolation: variable 'fs' has effect level Act, but closure is created in Pure context`

### 5.2 Effect-Produced Data (Blocked)

```ash
workflow w {
    let secret = std::fs::read_file("/etc/passwd");  -- Act-level value
    let helper = fn(x) { x + secret };  -- ERROR: captures Act-level value
    ret helper
}
```

**Error:** `CaptureEffectViolation: variable 'secret' has effect level Act (produced by std::fs::read_file)`

### 5.3 Higher-Stratum Closure Capture (Blocked)

```ash
workflow w {
    let act_fn = \|x\| -*> { ... };  -- Act closure (reserved syntax)
    let bad = fn(x) { act_fn(x) };  -- ERROR: captures Act closure in pure closure
    ret bad
}
```

**Error:** `CaptureEffectViolation: variable 'act_fn' has effect level Act`

### 5.4 Pure Capture (Allowed)

```ash
fn make_adder(n) {           -- pure context
    fn(x) { n + x }          -- ✅ n is Int, pure value
}

fn make_multiplier(factor) {  -- pure context
    fn(x) { x * factor }     -- ✅ factor is Int, pure value
}

fn compose(f, g) {           -- pure context
    fn(x) { f(g(x)) }        -- ✅ f and g are pure closures
}
```

## 6. Tower Callable Type Integration

### 6.1 Closure Type with Effect Annotation

Extend the closure type to carry the effect level of its captures:

```rust
Type::Closure {
    params: Vec<Type>,
    ret: Box<Type>,
    capture_effect: EffectLevel,  -- NEW: maximum effect of captured values
}
```

A pure closure has `capture_effect: Pure`. This allows the type system to track closure purity through the type.

### 6.2 Effect-Polymorphic Closures

A closure's effect level is the maximum of:
1. The effect level of its body
2. The effect level of its captures

```text
fn make_adder(n: Int) -> (Int) -> Int  -- pure closure, pure captures
fn make_reader(fs: Fs) -> (Path) -> String  -- ERROR: cannot create pure closure capturing Act value
```

## 7. Diagnostics

| Case | Diagnostic | Suggested Fix |
|------|-----------|-------------|
| Pure closure captures Act capability | `CaptureEffectViolation` | Pass capability explicitly as argument, or create closure in Act context |
| Pure closure captures effect-produced value | `CaptureEffectViolation` | Extract pure data before creating closure, or create closure in Act context |
| Pure closure captures higher-stratum closure | `CaptureEffectViolation` | Use function composition instead of closure capture |
| Closure effect level exceeds context | `ContextEffectMismatch` | Move closure creation to higher stratum, or reduce captures |

## 8. Acceptance Criteria

### C88-1: Pure closures with pure captures

```ash
fn make_adder(n: Int) -> (Int) -> Int {
    fn(x) { n + x }
}
let add5 = make_adder(5);
assert add5(3) == 8;
```

### C88-2: Reject capability capture in pure closures

```ash
fn make_reader(fs: Fs) -> (Path) -> String {
    fn(path) { fs.read(path) }  -- type error: captures Act-level value
}
```

### C88-3: Reject effect-produced value capture

```ash
fn make_secret_reader() -> (Int) -> String {
    let secret = std::fs::read_file("/etc/passwd");  -- Act
    fn(x) { secret }  -- type error: captures Act-level value
}
```

### C88-4: Closure effect tracked in type

```ash
let pure_fn: (Int) -> Int = fn(x) { x + 1 };  -- Type::Closure with capture_effect: Pure
```

### C88-5: Tower examples in documentation

Reference documentation shows closures at each stratum with correct capture rules.

## 9. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-031 | Amends: replaces blanket ban with capture-based rule |
| SPEC-072 | Integrates: closure stratum arrows align with capture effect levels |
| SPEC-087 | Enables: deferred QuickCheck combinators become implementable |
| SPEC-056 | Consistent: Workflow closures follow same capture rules |

## 10. Deferred Items

| Item | Reason | Future Work |
|------|--------|-------------|
| Mutable capture | Ash has no mutable refs | Add when mutable state is introduced |
| Recursive closure self-capture | Requires late binding analysis | Already works via `BindingSlot::Late` |
| Cross-stratum closure passing | Requires serialization | Deferred to process/workflow boundary spec |

## 11. Verification Strategy

1. **Typechecker tests:** Prove capture analysis accepts pure captures, rejects effect captures
2. **Runtime tests:** Verify fallback enforcement matches static analysis
3. **Property tests:** Generate random capture sets, verify effect-level monotonicity
4. **Documentation tests:** All tower examples parse and typecheck correctly
5. **Negative tests:** Each rejection scenario produces correct diagnostic

## 12. Implementation Notes

### ash-parser
- No changes needed: `Expr::FnDef` already parses in all contexts
- Diagnostic improvement: capture-specific error instead of generic "closure in pure context"

### ash-typeck
- Add `capture_effect` field to closure types
- Implement `extract_effect_level` for all types
- Check capture set at closure creation point

### ash-interp
- Replace `ctx.is_pure()` blanket check with `env_frame.is_pure_capture()`
- Or remove runtime check entirely if typechecker guarantees correctness

### ash-core
- Add `EffectLevel` enum to shared types
- Extend `EnvFrame` with capture effect metadata

## 13. Documentation Tasks

This spec enables the following documentation tasks:

- TASK-1525: Write `reference/language/functions.md` with closure syntax, capture rules, and tower examples
- TASK-1526: Write `reference/language/tower.md` with stratum examples and callable arrows
- TASK-1527: Update `reference/language/types/records.md` with closure field examples
- TASK-1528: Write cookbook examples for closures at each stratum

## 14. Closeout Criteria

- [ ] All C88-1 through C88-5 acceptance criteria pass
- [ ] Typechecker rejects all effect-capture violations
- [ ] Runtime enforces or trusts typechecker (no double-check needed)
- [ ] Documentation covers all four strata with examples
- [ ] PLAN-152 and PLAN-INDEX updated
- [ ] CHANGELOG.md records the refinement
- [ ] SPEC-031 and SPEC-072 amended with cross-references
