---
id: ref.language.functions.local
title: Local and Anonymous Functions
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-06-17
verified_against:
  git_commit: 41ebf740
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md
    - docs/plan/tasks/TASK-1525-reference-functions-and-closures.md
    - docs/plan/tasks/TASK-1527-update-record-docs-with-closure-fields.md
  code:
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-core/src/ast.rs
    - crates/ash-interp/src/eval.rs
    - crates/ash-core/src/value.rs
    - crates/ash-core/src/env_frame.rs
  tests:
    - crates/ash-interp/src/eval/tests.rs
  examples: []
related:
  depends_on:
    - ref.language.functions
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Local and Anonymous Functions

## Summary

Ash supports local function values in expression contexts. They are useful for small transformations and higher-order helpers, but they are not the same thing as module-level function exports.

## Anonymous `fn` expressions

An anonymous function expression starts with `fn`, has a parameter list, may have a return type, and has a body block.

```ash
pub fn use_local(n: Int) -> Int {
    let double = fn(x: Int) -> Int { x * 2 };
    double(n)
}
```

The parser represents this as a function-definition expression (`FnDef`) rather than as a module item.

## Named local functions

Inside a function body or block, a named local function desugars to a `let` binding of an anonymous function.

```ash
pub fn use_named_local(n: Int) -> Int {
    fn double(x: Int) -> Int { x * 2 }
    double(n)
}
```

This is equivalent in shape to:

```ash
pub fn use_named_local(n: Int) -> Int {
    let double = fn(x: Int) -> Int { x * 2 };
    double(n)
}
```

## Closure shorthand

For a short expression body, use closure shorthand:

```ash
pub fn use_shorthand(n: Int) -> Int {
    let double = |x| -> x * 2;
    double(n)
}
```

The shorthand desugars immediately to an anonymous `fn` expression and remains pure even when
written inside effectful target contexts. It does not carry a return-type annotation; use full
`fn(...) -> ... { ... }` syntax when the type needs to be explicit. The historical higher-stratum
closure arrows `|args| -*>`, `|args| =>`, and `|args| =*>` are reserved and rejected until those
callable semantics exist.

## Capture and the effect-safe rule

Local function values capture lexical variables from their surrounding scope. Ash enforces a **capture-based effect rule**: a closure created in context C may only capture values whose effect level ≤ C.

### Allowed captures

Pure closures may capture pure values (Int, String, Bool, records of pure values):

```ash
pub fn add_n(n: Int, x: Int) -> Int {
    let add = fn(y: Int) -> Int { n + y };
    add(x)
}
```

Record values with pure fields are also allowed:

```ash
pub fn scale_point(p: Point, factor: Int) -> Point {
    let scale = fn(coord: Int) -> Int { coord * factor };
    Point { x: scale(p.x), y: scale(p.y) }
}
```

### Rejected captures

A pure closure may **not** capture effectful values (capabilities, provider-backed handles, or closures outside the pure boundary). The runtime rejects these with a `CaptureEffectViolation` error:

```ash
-- REJECTED: capability capture in pure closure
pub fn make_reader(fs) {
    let read = fn(path) { fs.read(path) };  -- Error: fs requires a provider-backed effect row
    read("/tmp/data.txt")
}
```

```ash
-- REJECTED: effect-produced value capture
pub fn make_handler(secret) {
    let handler = fn(req) { process(req, secret) };  -- Error if secret was produced by an effectful boundary
    handler
}
```

### Effect levels

| Level | Values | Closure rule |
|-------|--------|-------------|
| Pure (0) | Int, Float, String, Bool, Null, pure records | Can be captured by any closure |
| Effectful runtime (1) | Provider-backed handles, streams, admitted runtime values | Can only be captured inside an owning target effect/profile boundary |
| Process (2) | Process handles and channel-owned values | Can only be captured inside process/channel-owned boundaries |
| Application (3) | Application instance metadata and control links | Can only be captured inside application runtime boundaries |

The typechecker currently types all closures as `Type::Fn` (pure). Full capture analysis in the typechecker is deferred; the runtime enforces the rule as the defense-in-depth safety net.

## Struct literal fields

Anonymous `fn` expressions and closures can be used as field values in struct literals. This is the primary pattern for constructing higher-order values like strategies or handlers.

```ash
pub fn make_handler() -> Handler {
    Handler {
        on_request: fn(req: Request) -> Response { process(req) },
        on_error: |err| -> default_response(),
    }
}
```

### Multi-field struct literal support

As of TASK-1510, anonymous `fn` expressions and closure shorthand are accepted in single-field and multi-field struct literals, including trailing-comma forms and generic return annotations on anonymous functions.

You may still use `let` bindings when that improves readability:

```ash
pub fn make_handler() -> Handler {
    let on_req = fn(req: Request) -> Response { process(req) };
    let on_err = |err| -> default_response();
    Handler {
        on_request: on_req,
        on_error: on_err,
    }
}
```

## Current boundaries

- Module-level functions are collected as definitions and imported by module machinery; they are not reified as serializable closure values.
- Local closures are not safe to send across process boundaries.
- Partial application is not supported. If a function expects two arguments, call it with two arguments.
