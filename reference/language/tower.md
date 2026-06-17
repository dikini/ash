---
id: ref.language.tower
title: The Ash Tower
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
    - docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md
    - docs/spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md
    - docs/spec/SPEC-091-LET-DESTRUCTORS.md
  code:
    - crates/ash-core/src/effect.rs
    - crates/ash-interp/src/eval.rs
    - crates/ash-typeck/src/type_env/interfaces_and_summary_types.rs
  tests:
    - crates/ash-interp/src/eval/tests.rs
  examples:
    - std/src/test/quickcheck/strategy.ash
    - std/src/test/quickcheck/combinator.ash
---

# The Ash Tower

## Summary

The Ash effect tower organizes code into four strata by computational power. Each stratum can do everything the ones below it can, plus additional operations that require more runtime authority.

```text
Pure < Act < Proc < Workflow
```

| Stratum | What it can do | Callable arrow |
|---------|---------------|----------------|
| **Pure** | Value computation, function calls, pattern matching | `(A) -> B` |
| **Act** | Pure + read external state, call capabilities | `(A) -*> B` (reserved) |
| **Proc** | Act + spawn processes, send/receive messages | `(A) => B` (reserved) |
| **Workflow** | Proc + obligations, policies, decisions, send to agents | `(A) =*> B` (reserved) |

## Pure

Pure functions compute values without side effects. They are the foundation of the tower.

```ash
pub fn add(a: Int, b: Int) -> Int {
    a + b
}
```

Pure closures are allowed if they only capture pure values:

```ash
pub fn make_adder(n: Int) -> (Int) -> Int {
    fn(x) { n + x }
}
```

### Pure callable type

The `(A) -> B` type is the only callable arrow currently available. Higher-stratum arrows are reserved for future use.

```ash
pub fn apply_twice<T>(x: T, f: (T) -> T) -> T {
    f(f(x))
}
```

## Act

The `Act` stratum reads external state and calls capabilities. It is the first effectful layer.

```ash
pub fn read_config(path: String) -> Act<String> {
    do:Act {
        return fs.read(path)
    }
}
```

Act closures may capture Act-level values (capabilities, streams):

```ash
pub fn make_reader(fs) -> (String) -> Act<String> {
    fn(path) {
        do:Act {
            return fs.read(path)
        }
    }
}
```

## Proc

The `Proc` stratum manages processes: spawning, sending messages, receiving.

```ash
pub fn spawn_worker(task: String) -> Proc<ProcessHandle> {
    do:Proc {
        return spawn worker(task)
    }
}
```

## Workflow

The `Workflow` stratum is the highest level: obligations, policies, decisions, agent communication.

```ash
pub fn approve_request(req: Request) -> Workflow<Decision> {
    do:Workflow {
        decide under policy::approval {
            allow -> return Approved
            deny -> return Rejected
        }
    }
}
```

## Crossing boundaries

### Pure to Act

A pure function can construct an `Act` value but cannot execute it:

```ash
-- Pure function returns Act value as data
pub fn make_greeting(name: String) -> Act<String> {
    do:Act {
        return "hello " + name
    }
}
```

### Act to Proc

An Act can construct a Proc value:

```ash
pub fn prepare_worker(config: String) -> Act<Proc<ProcessHandle>> {
    do:Act {
        return do:Proc {
            return spawn worker(config)
        }
    }
}
```

### Effect levels and capture

The capture-based effect rule ensures closures don't smuggle effectful values into pure contexts:

| Closure context | Can capture |
|-----------------|-------------|
| Pure | Pure values only |
| Act | Pure + Act values |
| Proc | Pure + Act + Proc values |
| Workflow | Any value |

Violations are rejected at runtime with `CaptureEffectViolation`:

```ash
-- REJECTED: pure closure capturing Act value
pub fn bad(fs) {
    let f = fn(path) { fs.read(path) };  -- Error: fs is Act-level
    f("/tmp/data.txt")
}
```

## Common patterns

### Strategy<T> (pure higher-order record)

The `Strategy<T>` type from `test::quickcheck` is a pure record with function fields:

```ash
pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
};
```

### Using destructuring with Strategy

```ash
pub fn map_strategy<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B> {
    let { gen, shrink } = s;
    Strategy {
        gen: fn(ctx) { f(gen(ctx)) },
        shrink: fn(b) { [] },
    }
}
```

## Known limitations

- Higher-stratum callable arrows (`-*>`, `=>`, `=*>`) are reserved but not yet implemented
- The typechecker types all closures as `Type::Fn` (pure); runtime enforces capture rules
- Cross-stratum closure serialization is not supported
- Partial application is not supported

## See Also

- [Functions and Pure Code](functions.md) — pure function syntax and boundaries
- [Local and Anonymous Functions](functions/local-and-anonymous.md) — closures and capture rules
- [Record Types](types/records.md) — records with function fields
- [SPEC-072: Tower Callable Type and Closure Syntax](../../docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../docs/spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
