# PLAN-099: Act-to-Proc Embedding Boundary

Goal: introduce the deferred `proc::from_act : Act<A> -> Proc<A>` surface promised by SPEC-048, but only after verifying the landed Phase 97 Act runtime boundary and preserving the semantic tower distinction between `Act<A>` and `Proc<A>`.

## Scope

This phase is a narrow post-Phase-98 follow-on. It does not reopen the completed Phase 98 process/workflow runtime slice. Instead, it adds the explicit embedding boundary from sequential effectful computation into process-structured computation.

In scope:
- verifying the exact landed Phase 97 `Act` force/hidden-carrier contract before implementation
- adding `proc::from_act` surface/type/runtime wiring
- proving the embedding is honest about `ActEnv`, lower-cause preservation, and process identity behavior

Out of scope:
- redefining `Proc<A>` as `Act<A>`
- exposing raw `ActEnv` as an Ash value
- adding new child-process, scheduling, or workflow-report semantics beyond the existing Phase 98 surface
- flattening `Proc<Act<A>>`

## Dependencies

- ✅ TASK-718 — `Proc` core `unit`/`bind`/`then` combinators
- ✅ TASK-689D — honest opaque public `Act` boundary and hidden-carrier force path
- ✅ TASK-690 — cross-layer validation for parse -> type -> execute

## Decision gates

### D1: Embedding direction is explicit, not implicit

`from_act : Act<A> -> Proc<A>` is the only surface introduced here. `Proc<Act<A>>` remains distinct and does not implicitly flatten.

### D2: Hidden `ActEnv` boundary remains protected

The task must reuse the verified hidden-carrier force path rather than exposing `ActEnv` structure or accepting visible fake carriers as sufficient proof.

### D3: No accidental process-semantics inflation

If `from_act` returns a `Proc<A>` that is forced inside the current process context, it must not silently allocate child `ProcessId`s, public `P<A>` handles, or workflow-boundary reports unless that behavior is explicitly added and verified.

## Task list

### TASK-719

Verify and expose `proc::from_act` as the Act-to-Proc embedding boundary.

## Recommended execution order

1. Re-read TASK-718, TASK-689D, TASK-690, SPEC-047, SPEC-048, and SPEC-049.
2. Write failing tests proving that `proc::from_act` is currently absent and that the hidden `Act` runtime contract matters.
3. Add the narrow stdlib/type/runtime surface for `proc::from_act`.
4. Verify the embedding preserves semantic-tower boundaries and does not overclaim concurrency/process semantics.
5. Run focused and workspace verification gates.

## Verification gates

After each implementation task:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- targeted task-specific `cargo test` commands listed in the task file
- update `CHANGELOG.md` for implementation/tooling/docs-policy changes

Phase-close verification:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- focused end-to-end tests covering `proc::from_act` typing and forcing behavior
- doc drift audit against `docs/spec/SPEC-047-ACT-MONAD.md`, `docs/spec/SPEC-048-PROC-LIBRARY.md`, and the new task file

## Expected deliverable

A narrow, honest embedding surface where:
- `proc::from_act` is publicly available and typed as `Act<A> -> Proc<A>`
- the returned `Proc<A>` forces through the verified hidden `ActEnv` runtime path
- `Act<A>` and `Proc<A>` remain distinct public strata
- no fake `ActEnv` carrier or accidental child-process semantics leak through the embedding
