# NOTE-005: The Act Monad — Unifying Pure and Effectful Computation

**Date:** 2026-04-21
**Status:** Open — design exploration
**Priority:** High — resolves NOTE-004, foundational for fn/capability/workflow unification
**Supersedes:** NOTE-004 (partial — NOTE-004's open questions are answered here)
**Depends on:** SPEC-020 (ADTs, Kind system), SPEC-033 (multi-parameter interfaces), SPEC-035 (associated types)

## 1. Problem

Ash has two non-composable evaluation contexts:

| Context    | Entry point          | What it does                              | What it can't do            |
|------------|----------------------|-------------------------------------------|------------------------------|
| Expression | `eval_expr(expr, ctx)` | Pure evaluation, no traces, no providers  | Call capabilities, log effects |
| Workflow   | `execute_workflow(wf, ctx, cap_ctx, ...)` | Effectful execution, traces, providers | Return first-class effectful values |

The gap: you cannot call an effectful operation from inside an expression, and you cannot use expression-level results as first-class things in the workflow context without the awkward `Orient` bridge. The stdlib `.ash` files contain `act execute` inside `fn` bodies — syntax that cannot currently parse, because `act` only exists as a `Workflow` node, not an `Expr` variant.

## 2. Proposal

Introduce `Act` as a first-class type constructor in the expression layer.

```
type Act<A> = ActEnv → Result<(A, ActEnv), ExecError>
```

`Act a` is a computation that, given an environment containing providers, policies, and provenance, may produce a value of type `a` alongside an accumulated effect log, or fail with an error.

The four primitive operations:

```
unit   : a → Act a                                     -- lift pure value
bind   : Act a → (a → Act b) → Act b                   -- sequence computations
invoke : (Provider, Action, [Value]) → Act Value        -- primitive effect
guard  : Policy → Act a → Act a                         -- push policy onto stack
```

## 3. The Threaded Environment

```
ActEnv = {
  capability_ctx : CapabilityContext,    -- provider registry (read-only during execution)
  policies       : PolicyStack,          -- what's allowed (composed, stack-scoped)
  provenance     : Provenance,           -- audit trail seed (who/where/why)
  effects        : EffectLog,            -- append-only log of effects that occurred
}
```

Properties of the threaded environment:

1. **Sequential ordering**: Effects appear in execution order. `bind` threads left-to-right.
2. **Provenance propagation**: Each effect records its provenance chain. Nested calls trace through callers.
3. **Append-only effect log**: No computation can remove effects. The audit trail is tamper-proof.
4. **Provider immutability**: Providers are registered at engine build time. No capability escalation during execution.
5. **Policy stacking**: Guards compose. Every `invoke` passes through the full policy stack. There are no unguarded effects.
6. **Failure short-circuits**: `bind` on `Err` skips the continuation. The effect log only contains effects that actually happened.

## 4. All Acts Are Guarded

Every `invoke` is policy-checked. This is not optional. The `where` guard in workflow syntax:

```
act provider:action(args) where guard_expr
```

is sugar for an inline policy — a predicate composed into the policy chain:

```
guarded_invoke(extra_policy, invoke(provider, action, args))
  = |env| => invoke(provider, action, args)(env.with_pushed_policy(extra_policy))
```

There is no code path that bypasses policy. `unit` doesn't need policy (it produces no effects). `bind` doesn't need policy (it threads, doesn't act). Only `invoke` checks policy.

## 5. Surface Syntax

### 5.1 Act blocks

```
act {
    x = read(path)                -- bind  (RHS is Act String, x : String)
    n = len(x)                    -- inline (RHS is Int, pure substitution)
    y = parse(x)                  -- bind  (RHS is Act Value, y : Value)
    ret (x, n, y)                 -- unit  (wraps pure tuple)
}
```

The `=` operator in act blocks is overloaded by type:
- If RHS : `Act a` → `bind`, binding unwraps to `a`
- If RHS : `a` → pure substitution (no monadic step, inlined in lambda body)

Desugaring:

```
act {
    x = read(path)
    n = len(x)
    y = parse(x)
    ret (x, n, y)
}

-- becomes:

bind(read(path), |x|
  bind(parse(x), |y|
    unit((x, len(x), y))))
```

Note: `n` does not appear in the desugared form. `len(x)` is inlined directly in the tuple. Pure bindings are syntactic convenience, not monadic operations.

### 5.2 Effectful function declarations

The return type declares the effect:

```
-- Pure function
fn concat(a: String, b: String) -> String { string::concat(a, b) }

-- Effectful function
fn read(path: String) -> Act String {
    act {
        result = invoke(Fs, "read", [path])
        ret result
    }
}

-- Effectful function using another effectful function
fn process(path: String) -> Act (String, String) {
    act {
        content = read(path)
        filtered = filter(|c| => c != ' ', content)
        ret (content, filtered)
    }
}
```

### 5.3 Monadic bind as expression

The `act {}` block is sugar. `bind` is also available directly:

```
bind(read(path), |content|
  bind(parse(content), |result|
    unit(result)))
```

### 5.4 Monad laws (semantic correctness)

```
-- Left identity
bind(unit(a), f)  =  f(a)

-- Right identity
bind(m, unit)     =  m

-- Associativity
bind(bind(m, f), g)  =  bind(m, |x| => bind(f(x), g))
```

Proof obligations:
- `unit(v) = |env| => Ok((v, env))` — identity on environment
- `bind` threads left-to-right, no reordering
- Effect log concatenation is associative; empty log is identity

## 6. Function Classification Under Act

| Declaration           | Type            | Body can contain   | Example                           |
|-----------------------|-----------------|--------------------|------------------------------------|
| `fn f(x: A) -> B`     | `A → B`         | pure expr only     | `fn len(s) -> Int`                |
| `fn f(x: A) -> Act B` | `A → Act B`     | `act {}` blocks    | `fn read(p) -> Act String`        |
| `builtin fn f(...) -> B` | `A → B`      | Rust body          | `builtin fn concat(...) -> String` |
| capability dispatch   | `A → Act B`     | provider invoke    | (declared, no Ash body)           |

The purity boundary moves from runtime (`ctx.is_pure()`) to the type system: a function returning `a` cannot contain `act {}` blocks. A function returning `Act a` can.

## 7. Relationship to Workflow

`Workflow` becomes a structured subset of `Act`. OODA phases are combinators:

```
observe cap pattern then rest
  ≈ bind(invoke_observe(cap), |result|
       match(result, pattern, rest))

act provider:action(args) as name then rest
  ≈ bind(invoke(provider, action, args), |result|
       ... with name bound to result ...)

orient expr then rest
  ≈ bind(unit(expr), |_ignored| rest)
```

The Workflow syntax adds guard checking, policy evaluation, and provenance tracking on top of bare `bind`. A bare `act {}` block is the lightweight path — monadic composition without OODA ceremony.

## 8. Governance and Effects

Effects appear only in the RHS of semantic rules, via `invoke`:

```
invoke(provider, action, args) = |env| =>
  match env.policies.check(provider, action, args, env.provenance):
    Deny(reason) => Err(PolicyViolation(reason))
    Allow =>
      let result = env.capability_ctx.execute(provider, action, args)
      let effect = Effect { provider, action, args, result,
                            effect_type, provenance: env.provenance }
      Ok((result, env.with_appended_effect(effect)))
```

The effect log is the audit trail. Only `invoke` entries appear in it. `unit` adds nothing. `bind` concatenates. Governance inspects the log at any point.

## 9. Encoding in Ash (No Runtime Magic)

The entire monad is expressible as ordinary Ash functions:

```
fn unit(v: a) -> Act a {
    |env| => Ok((v, env))
}

fn bind(ma: Act a, f: (a → Act b)) -> Act b {
    |env| => match ma(env) {
        Ok((a, env')) => f(a)(env'),
        Err(e) => Err(e)
    }
}

fn then(ma: Act a, mb: Act b) -> Act b {
    bind(ma, |_a| => mb)
}

fn guard(policy: Policy, ma: Act a) -> Act a {
    |env| => match env.policies.check(policy) {
        Deny(reason) => Err(PolicyViolation(reason)),
        Allow => ma(env)
    }
}
```

The single runtime primitive:

```
builtin fn invoke(provider: String, action: String, args: List) -> Act Value
```

Everything else is library code. `Act a` is a type alias for a function type. `bind` is a higher-order function.

Bootstrapping layers:

```
Runtime provides:  invoke, ActEnv construction
Library provides:  unit, bind, then, guard, act {} sugar
Type system knows: Act a ≈ ActEnv → Result<(a, ActEnv), ExecError>
```

## 10. Interface Hierarchy (Future)

The Functor → Applicative → Monad hierarchy can be expressed once the type system supports interfaces over type constructors (`* → *` kind bounds on interface parameters).

Current status:
- SPEC-020 defines `Kind::Type` and `Kind::Arrow` — the kind system exists
- SPEC-033 defines multi-parameter interfaces
- SPEC-035 defines associated types
- Missing: associated type constructors (`type F<A>`) and kind bounds on interface params

Near-term: use `map_act`, `pure_act`, `bind_act` as library functions.
When type system matures:

```
interface Monad<M> where M: * → * {
    fn bind<A, B>(ma: M<A>, f: (A → M<B>)) → M<B>
    fn unit<A>(a: A) → M<A>
}

impl Monad for Act {
    bind(ma, f) = bind_act(ma, f)
    unit(a)     = pure_act(a)
}
```

## 11. Design Choices Not Resolved

1. **observe vs execute at the type level** — `Act a` doesn't distinguish effect types. If governance needs type-level distinction, options: `Observe a` / `Execute a` as separate types, or phantom type parameter `Act<Eff, a>`.

2. **Concurrent composition** — `bind` is sequential. `par : Act a → Act b → Act (a, b)` is a valid future combinator that runs computations concurrently and merges effect logs.

3. **`act` vs `do` vs `perform`** — the keyword for the block sugar is undecided. The construct is the same regardless of surface name.

4. **Migration** — existing `.ash` stdlib files with `act execute` inside `fn` bodies would need return type annotations changed to `Act T`. This note deliberately ignores migration concerns.

## 12. What This Resolves

| NOTE-004 Open Question                          | Resolution in this note                    |
|-------------------------------------------------|---------------------------------------------|
| Should `builtin fn` be pure-only?               | Yes. Effectful ops return `Act a`.          |
| Should `Effect` gate constructs?                | The type `Act a` is the gate.               |
| Is observe/execute distinction sufficient?      | Deferred — see §11.1.                      |
| How do `extern fn` and `builtin fn` relate?     | Both are `a → b`. Effectful FFI → `a → Act b`. |
| Should workflows have effect annotations?       | Workflow IS an effect-annotated Act computation. |

## 13. Typing Rules (Formal)

### Effectful binding

```
Γ ⊢ e : Act a     Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────
Γ ⊢ act { x = e; rest } : Act b
  = bind(e, λx. act { rest })
```

### Pure binding (auto-unit, no monadic step)

```
Γ ⊢ e : a         Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────
Γ ⊢ act { x = e; rest } : Act b
  = bind(unit(e), λx. act { rest })
  -- but e is inlined: the unit/bind pair collapses
  -- because unit(v)(env) = (v, env) and bind on that
  -- just calls f(v)(env), so this is f(v)(env)
```

### Return

```
Γ ⊢ e : a
──────────────────────
Γ ⊢ act { ret e } : Act a
  = unit(e)
```

### Invoke

```
──────────────────────────────────────────────────────
Γ ⊢ invoke(provider, action, args) : Act Value
  (well-formed when provider is registered, action is known)
```

### Guard

```
──────────────────────────────────────
Γ ⊢ guard(p, ma) : Act a
  (when ma : Act a, p : Policy)
```

### Purity enforcement

```
Γ ⊢ fn f(x: A) -> B { body }
──────────────────────────────
body must not contain act {} blocks
(i.e., no sub-expression has type Act _)

Γ ⊢ fn f(x: A) -> Act B { body }
──────────────────────────────────
body may contain act {} blocks
```
