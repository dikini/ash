# DESIGN-020: Pure Functions and the Three-Vertex Model

## Status: Draft

## Overview

Introduce `fn` as a first-class construct for pure computation in Ash, establishing a three-vertex
model: **fn** (pure transform), **capability** (effect contract), **workflow** (orchestration).
This design addresses the gap between the aspirational std/ library syntax and the current parser,
and establishes a clean separation between pure computation and effectful orchestration.

## Problem Statement

The `std/` library files use `pub fn`, `match`, `if/else`, `panic`, `Fun(T) -> U`, and other
constructs that the current parser cannot handle. The stdlib_parsing.rs tests are ~70% string-matching
and do not validate these constructs through the parser.

Analysis revealed that the gap is not merely missing parser features -- it reflects a missing
language concept. Ash has workflows (orchestration) and capabilities (effect contracts) but no
construct for pure data transformation. The std/ files were written in an aspirational syntax that
assumes this third category exists.

## Design Decisions

### D1: Three-Vertex Model

Ash programs are composed from three distinct vertices:

```
         Transform (pure)
         fn -- total, deterministic
        / \
       /   \
      /     \
Orchestrate  Effect (capability)
workflow     observe/execute
temporal      provider-bound
```

Composition rules:
- fn -> fn (freely composes)
- workflow -> fn (workflow calls fn for data transforms)
- workflow -> cap (workflow uses capabilities for effects)
- fn -X-> workflow (functions never invoke workflows)
- fn -X-> cap (functions never use capabilities)

This is analogous to Erlang/OTP: processes (workflows) call pure functions for data plumbing.
Functions don't become processes.

### D2: Unified Surface Syntax, Split Semantics

The syntax for `let`, `if ... then ... else`, `match`, function calls, constructors, and operators
is shared between fn and workflow bodies. The keyword at the top (`fn` vs `workflow`) sets the
evaluation mode. The compiler enforces the boundary.

Mode-specific syntax:
- **fn only**: tail-expression return (no keyword needed), `panic` for fatal errors
- **workflow only**: `ret`, `act`, `receive`, `send`, `spawn`, `oblige`, `check`, `maybe`, `must`,
  `attempt`, `retry`, `timeout`, `observe`/`orient`/`propose`/`decide` phases
- **shared**: `let`, `if ... then ... else`, `match`, function calls, constructors, operators

fn calls use `module::name(args)` (double colon) syntax, distinct from capability calls
which use `provider:action(args)` with single colon.

### D3: Tail-Expression Return in fn, Explicit `ret` in Workflows

fn bodies use tail-expression return: the last expression in a block is the return value. No `ret`
keyword. Workflow bodies use explicit `ret expr`. The absence of `ret` in fn is itself a mode signal:
"you're in pure territory, just compute a value." The presence of `ret` in workflows marks a
lifecycle event -- control link resolution, provenance capture, obligation finalization.

### D4: fn Body Compiles to Expr Nodes

Under the hood, fn bodies compile to `Expr` AST nodes and workflow bodies compile to `Workflow`
AST nodes. This is an implementation detail, not a developer concern. The separation exists because:
- Expr nodes have no continuation chains, effect levels, or provenance spans
- Workflow If produces steps; Expr If produces values with branch type agreement
- Match arms in Expr produce values; in Workflow they produce workflow steps

### D5: fn Contracts

fn supports `requires`/`ensures` contracts, but only the arithmetic subset:
- `requires: n >= 0` (Arithmetic constraints)
- `ensures: result > 0` (postcondition predicates)

fn contracts explicitly exclude:
- `HasCapability` -- fn has no capabilities
- `HasRole` -- fn has no authority context
- `oblige`/`check` obligations -- fn has no lifecycle

fn contracts use the same syntax (`requires`/`ensures`) and the same core Contract struct from ash_core, but the surface Requirement::Arithmetic (which carries a raw Expr) needs a lowering pass to the core Requirement::Arithmetic (which carries structured var+constraint). This lowering pass is a prerequisite task (see PLAN-023).

### D6: Constraint Evolution Path

The constraint system evolves in stages:

| Stage | What | Checking |
|-------|------|----------|
| Current | `ArithConstraint` (Gt/Lt/Eq/Range on i64) | Runtime |
| Near term | `ValueConstraint` adding string predicates (StartsWith, Contains, MinLength) | Runtime |
| Z3 integration | Mixed integer + string theory behind `smt` feature flag | Compile time |
| Future | Dependent constraints (`n < len(list)`) with sized types | Compile time |

Each stage is additive. fn inherits all stages because fn contracts are value-only.

### D7: Capability Composition via Workflows

Workflows can satisfy capability contracts by composing other capabilities + pure functions.
No new `on` handler syntax is needed -- the existing `receive` construct handles mailbox dispatch.
A workflow declares it implements a capability:

> **Note:** The following example is non-normative pseudocode illustrating the intended direction for workflow-implemented capabilities. The `implements` clause and loop+receive handler syntax are deferred design work.

```
workflow cache_impl(store: cap Store) implements Cache {
    loop {
        receive {
            Cache.fetch(id) as value => { ... },
            Cache.store(id, value) => { ... }
        }
    }
}
```

### D8: Proxy Collapse (Deferred)

The `proxy` construct semantically overlaps with "workflow that implements a capability."
Collapsing proxy into `workflow implements Cap` using existing `receive` is the intended direction.
What matters is that a workflow can act for a role. The mechanism for expressing this is a
separate concern, deferred to later work.

### D9: std/ Library Restructuring

Under the three-vertex model, the std/ library splits cleanly:

| Module | Construct | Rationale |
|--------|-----------|-----------|
| option.ash, result.ash | `fn` definitions | Pure transforms |
| io/path.ash (pure ops) | `fn` definitions | String manipulation, no IO |
| io/fs.ash, io/dir.ash, io/stdio.ash | `capability` + `workflow` | Side-effectful IO |
| io/meta.ash | `capability` declaration | Metadata queries |
| runtime/supervisor.ash | `workflow` | Effect orchestration |

### D10: Purity Enforcement

The type checker (or a validation pass) rejects effectful constructs inside fn bodies:
- Rejected: ret, act, receive, send, spawn, oblige, check, observe, orient, propose, decide,
  maybe, must, attempt, retry, timeout
- Accepted: let, if/then/else, match, function calls, constructors, operators, panic, tail-expression return

## Examples

### Pure function

```ash
fn classify(n: Int) -> String {
    if n > 0 then "positive"
    else if n < 0 then "negative"
    else "zero"
}
```

### Function with contract

```ash
fn sqrt(n: Float) -> Float
    requires: n >= 0
    ensures: result >= 0
{
    compute_sqrt(n)
}
```

### Function using match

```ash
fn is_some<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => true,
        None => false
    }
}
```

### Workflow calling function

```ash
workflow classify_and_log(n: Int, log: cap Logger) -> String {
    let label = classify(n);
    act log:info(label);
    ret label;
}
```

### Workflow implementing a capability (deferred)

```ash
-- Future direction, not in initial implementation
workflow cache_impl(store: cap Store) implements Cache {
    loop {
        receive {
            Cache.fetch(id) as value => {
                let key = cache_key("item", id);
                let result = act store:get(key);
                ret result;
            },
            Cache.store(id, value) => {
                let key = cache_key("item", id);
                act store:set(key, value);
                ret Done;
            }
        }
    }
}
```

## Impact on Existing Specs

**Note:** fn support is a cross-cutting feature affecting parser, AST, name resolver, type checker, and interpreter. It is **not** merely a parser change.

| Spec | Impact | Component Areas |
|------|--------|-----------------|
| SPEC-002 (Surface) | New `fn` construct, `match`/`if` expression forms, `panic` keyword | Parser, AST |
| SPEC-003 (Type System) | Function type syntax, FnType in type system, generic fn params, purity checking | Type checker |
| SPEC-004 (Semantics) | Fn evaluation rules, no effect/trace/provenance for fn | Interpreter |
| SPEC-009 (Module System) | fn as module-level definition, module::name call resolution, grammar update | Parser, Resolver |
| SPEC-012 (Imports) | fn import/export via `use`, `pub fn` visibility | Resolver |
| SPEC-020 (ADT Types) | Pattern matching in fn bodies | Parser, Type checker |
| SPEC-022 (Workflow Typing) | Contract subset rules for fn, fn precondition propagation | Type checker |
| New SPEC-027 | Pure Functions -- normative spec for fn construct | All components |
| New SPEC-028 | Fn Contract System -- constraint vocabulary for fn | Type checker, Constraint system |

## Open Questions

1. Should `panic` be a keyword or a built-in function? (Keyword preserves the option of special
   compilation; built-in function is simpler)
2. Should fn support recursion? (Yes, but needs termination checking for ensures proving --
   deferred)
3. Interaction between fn generics and capability generics -- are they the same mechanism?
   (Likely yes, but needs spec work)
   Resolution: fn types are pure and carry no effect slot. They do not use Type::Fun's effect parameter. See SPEC-027 §3.2.

## References

- SPEC-002: Surface Language
- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-020: Algebraic Data Types
- SPEC-022: Workflow Typing with Constraints
- SPEC-023: Proxy Workflows (proxy collapse context)
- DESIGN-019: Action Result Binding and Continuation
