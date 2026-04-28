# TASK-748: Do-Target Kinding and Dictionary Resolution

## Status: ✅ Complete

## Description

Add typechecker support for resolving `do:K` targets and MVP Act/Proc builtin dictionaries. This task does not implement full user-defined constructor-kinded `Monad<M>`; it creates a bridge shaped like future Monad evidence.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §§6-7
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)

## Dependencies

- ✅ TASK-746.
- ✅ TASK-747 parser/surface substrate.

## Requirements

1. Resolve `DoTarget` names through type-constructor resolution.
2. Accept MVP targets `Act` and `Proc` only.
3. Reject proper types such as `Int` with a kind diagnostic.
4. Reject unknown targets.
5. Resolve a hidden dictionary containing target constructor, return operation, bind operation, and tower level.
6. Do not import target-specific ordinary operations into lexical scope.
7. Preserve a TODO/deferred hook for future `Monad<M>` interface resolution.

## TDD Steps

### Step 1: Add failing typechecker tests

**Files:**

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify or create helper module in `crates/ash-typeck/src/` if needed.

Tests:

- `do:Act { return 1 }` target resolves to Act dictionary.
- `do:Proc { return 1 }` target resolves to Proc dictionary.
- `do:Int { return 1 }` reports wrong kind / not a computation constructor.
- `do:Missing { return 1 }` reports unknown do target.
- `do:Result { return 1 }` is rejected in MVP with a hint about deferred `Result<_, E>`.

### Step 2: Add dictionary model

Introduce a private typechecker data structure equivalent to:

```rust
enum DoTowerLevel { Effectful, Proc }

struct DoDictionary {
    target: QualifiedName,
    value_constructor: QualifiedName,
    return_op: QualifiedName,
    bind_op: QualifiedName,
    tower_level: DoTowerLevel,
}
```

Use exact project types where appropriate.

### Step 3: Wire target resolution

Implement helper(s) such as:

```rust
resolve_do_target(env: &TypeEnv, target: &DoTarget) -> Result<DoDictionary, ConstructorError>
```

Expected MVP mappings:

- `Act` -> Act dictionary.
- `Proc` -> Proc dictionary.

### Step 4: Verify

Run:

```bash
cargo test -p ash-typeck do_target -- --nocapture
cargo fmt --check
```

## Verification Steps

- [x] Target resolution tests pass.
- [x] Unknown and wrong-kind diagnostics are tested.
- [x] No user-defined `Monad<M>` support is overclaimed.
- [x] No Phase 104 files are touched.
- [x] Independent review confirmed dictionary shape can migrate to real `Monad<K>` evidence later after fixes for AST-only type kinding and hidden Act operation evidence.

## Completion Notes

- Added a private typechecker `do_target` substrate for resolving `DoTarget` to an MVP hidden dictionary.
- Accepted `Act` and `Proc` as the only MVP targets.
- Represented Act sequencing operations as hidden compiler evidence rather than ordinary lexical/value-env names.
- Represented Proc sequencing operations as existing ordinary `proc::unit` / `proc::bind` evidence.
- Added kind diagnostics for proper types, unknown-target diagnostics, and explicit deferred diagnostics for `Result<_, E>` and explicit target arguments.
- Added resolver-level and `check_expr` integration coverage proving valid targets reach the TASK-749 typed-elaboration boundary while invalid targets report target-resolution diagnostics.
- Fixed resolver kinding to use AST `TypeDef` parameter arity when a known type has not yet been converted into `TypeInfo`.

## Dependencies for Next Task

Required by:

- TASK-749: typed do elaboration.
