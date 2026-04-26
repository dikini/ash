# TASK-719: Verify and expose `proc::from_act` as the Act-to-Proc embedding boundary

## Status: ✅ Complete

## Description

TASK-718 intentionally deferred `proc::from_act` until the exact Phase 97 `Act` force/hidden-carrier substrate was implemented and verified. That substrate is now present through the opaque public `Act` boundary and cross-layer validation work. This task introduces the explicit `proc::from_act : Act<A> -> Proc<A>` surface promised by SPEC-048 while preserving the semantic-tower distinction between `Act<A>` and `Proc<A>`, keeping `ActEnv` runtime-only, and avoiding accidental process/runtime inflation.

## Specification Reference

- SPEC-047 §2.5
- SPEC-047 §7
- SPEC-048 §1.1
- SPEC-048 §3
- SPEC-049 compatibility constraints for process/runtime semantics

## Dependencies

- ✅ TASK-718: `Proc` core `unit`/`bind`/`then` combinators
- ✅ TASK-689D: honest opaque public `Act` boundary and hidden-carrier force path
- ✅ TASK-690: cross-layer validation for parse -> type -> execute

## Requirements

### Functional Requirements

1. Verify and document the exact landed Phase 97 `Act` force contract before implementation, including the hidden runtime `ActEnv` requirement and the protected visible-token boundary.
2. Add the public proc-library surface for `proc::from_act : Act<A> -> Proc<A>` without redefining `Proc<A>` as `Act<A>`.
3. Wire `proc::from_act` through stdlib, typechecking, and interpreter/runtime layers.
4. Ensure forcing the resulting `Proc<A>` reuses the verified hidden-`ActEnv` path honestly rather than exposing `ActEnv` as an Ash value or accepting visible fake carriers.
5. Preserve semantic distinctions: `Proc<Act<A>>` does not implicitly flatten, and `from_act(ma)` is distinct from leaving `ma` as an `Act<A>` result.
6. Preserve Phase 98 process semantics unless explicitly changed: this task must not silently allocate child `ProcessId`s, create public `P<A>` handles, or alter workflow-boundary reporting.
7. Preserve lower-cause and effect/process attribution honestly when an embedded `Act` fails during Proc forcing.
8. Document any async-only limitation or helper-runtime bridge dependency explicitly if the implementation cannot support all existing Proc force contexts.

### Property Requirements (proptest)

```rust
// Prefer focused regression tests unless the implementation exposes a stable
// algebraic invariant worth property coverage. This task is primarily about an
// honest embedding boundary and hidden-carrier/runtime semantics.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests for the missing proc surface and the intended embedding contract.

Suggested files:
- `crates/ash-interp/tests/task_719_proc_from_act_runtime.rs`
- `crates/ash-engine/tests/task_719_proc_from_act_stdlib.rs`
- `crates/ash-typeck/tests/task_719_proc_from_act_types.rs`

Red expectations:
- `proc::from_act` is not yet importable/typed/executable.
- A returned `Proc<A>` must not succeed through a visible fake carrier alone.
- The task must prove `from_act` does not imply `Proc<Act<A>>` flattening.

### Step 2: Implement (Green)

Implement the minimal embedding surface needed to satisfy the tests.

Likely touch points:
- `std/src/proc.ash`
- `std/src/lib.ash`
- `crates/ash-typeck/src/type_env.rs`
- `crates/ash-interp/src/eval.rs`
- `crates/ash-interp/src/context.rs`
- `crates/ash-interp/src/act_env.rs`
- `crates/ash-engine/src/lib.rs` (only if the embedding requires explicit engine-path bridging)

### Step 3: Integration (Green)

Wire the feature through real import/type/runtime paths honestly.

Integration checks must prove:
- `proc::from_act` is available through the stdlib/module path.
- Typed examples using `Act<A> -> Proc<A>` pass.
- Forcing the resulting `Proc<A>` exercises the existing hidden `ActEnv` path rather than bypassing it.
- Existing `proc::unit`/`bind`/`then`, process handle, and workflow-boundary behavior remain compatible.

### Step 4: Verification

Add or extend focused tests for:
- honest hidden-carrier enforcement
- no implicit `Proc<Act<A>>` flattening
- preserved lower failure cause / attribution at the embedding boundary
- no accidental child-process admission or handle creation

## Verification Steps

- [x] `proc::from_act` is importable and typed as `Act<A> -> Proc<A>`.
- [x] Focused tests prove the embedding reuses the verified hidden `ActEnv` path.
- [x] Focused tests prove visible fake carriers remain insufficient.
- [x] Focused tests prove `Proc<Act<A>>` does not implicitly flatten.
- [x] Focused tests prove no child `ProcessId`/public `P<A>` surface is added by this task unless explicitly documented.
- [x] `cargo test --all` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes cleanly.
- [x] `cargo fmt --check` passes.

## Completion Notes

- Added the explicit stdlib surface `proc::from_act<A>(ma: Act<A>) -> Proc<A>` in `std::proc` and registered it in `ash-typeck` as exactly `Act<A> -> Proc<A>`.
- Implemented the runtime embedding in `ash-interp` as an opaque `Proc` closure over `__proc_env` that forces the captured `Act` through the existing hidden `__act_env` path and projects the compatibility payload back out, preserving the public `Act`/`Proc` distinction.
- Focused tests now prove:
  - explicit embedding vs `Proc<Act<A>>` non-flattening,
  - hidden-carrier enforcement against visible fake carriers,
  - no child-process/public-handle inflation,
  - workflow-boundary preservation as a returned `Proc` closure value,
  - structured failing-Act propagation as `EvalError::OperationalFailure(...)` with Effectful/effect-scope attribution at the lower Act boundary.
- No async-only limitation note was required for the landed slice; the implemented force path is the existing async Proc-forcing route already used by Phase 98 Proc runtime tests.

## Dependencies for Next Task

This task outputs the explicit Act-to-Proc embedding boundary promised by SPEC-048. Downstream Proc/workflow tasks may depend on it when they need to lift sequential effectful computation into Proc composition without collapsing the `Act`/`Proc` distinction.

## Notes

- Preserve `Pure < Effectful / Act < Proc < Workflow` as a real semantic-tower distinction.
- Do not expose raw `ActEnv` structure or promote the internal Act force-result compatibility shape into public language semantics.
- If the implementation must choose between eager Act execution and lazy Proc forcing, state and test that choice explicitly.
- If the implementation only supports async Proc force paths honestly, record that limitation rather than silently using an unsound sync shortcut.
- Update `CHANGELOG.md` for implementation/tooling/docs-policy changes made while completing this task.
